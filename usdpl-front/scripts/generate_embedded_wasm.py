import base64

if __name__ == "__main__":
    print("Embedding WASM into udspl_front.js")
    # assumption: current working directory (relative to this script) is ../
    # assumption: release wasm binary at ./pkg/usdpl_bg.wasm
    with open("./pkg/usdpl_front_bg.wasm", mode="rb") as infile:
        with open("./pkg/usdpl_front.js", mode="ab") as outfile:
            outfile.write("\n\n// USDPL customization\nconst encoded = \"".encode())
            encoded = base64.b64encode(infile.read())
            outfile.write(encoded)
            outfile.write("\";\n\n".encode())
            outfile.write(
"""function asciiToBinary(str) {
  if (typeof atob === 'function') {
    return atob(str)
  } else {
    return new Buffer(str, 'base64').toString('binary');
  }
}

function decode() {
  var binaryString =  asciiToBinary(encoded);
  var bytes = new Uint8Array(binaryString.length);
  for (var i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return (async function() {return new Response(bytes.buffer);})();
}

export function init_embedded() {
    return init(decode())
}
""".encode())
    with open("./pkg/usdpl_front.d.ts", "a") as outfile:
        outfile.write("\n\n// USDPL customization\nexport function init_embedded();\n")
    print("Done: Embedded WASM into udspl_front.js")
