window = 'fix stdweb';
importScripts('js/app.js');

let reader = new FileReaderSync();

onmessage = async function(e) {
  let id = e.data.id;
  let file = e.data.file;
  let opts = e.data.opts;

  let wasm = await Rust.asstosrt_wasm;
  console.log("convert " + file.name);
  console.log(opts);

  try {
    let ass = reader.readAsArrayBuffer(file);
    let srt = wasm.assToSrt(ass, opts);
    console.log(srt);
    let srtUrl = URL.createObjectURL(srt);
    postMessage({id: id, srtUrl: srtUrl});
  } catch (e) {
    postMessage({id: id, error: e});
  }
}
