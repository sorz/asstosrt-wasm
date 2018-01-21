window = 'fix stdweb';
importScripts('js/app.js');

let reader = new FileReaderSync();

onmessage = async function(e) {
  let id = e.data.id;
  let file = e.data.file;
  let inCharset = e.data.inCharset;
  let outCharset = e.data.outCharset;

  let wasm = await Rust.asstosrt_wasm;
  console.log("convert " + file.name);

  try {
    let ass = reader.readAsArrayBuffer(file);
    let srt = wasm.assToSrt(ass, inCharset, outCharset);
    console.log(srt);
    let srtUrl = URL.createObjectURL(srt);
    postMessage({id: id, file: file, srtUrl: srtUrl});
  } catch (e) {
    postMessage({id: id, file: file, error: e});
  }
}
