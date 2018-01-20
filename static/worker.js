window = 'fix stdweb';
importScripts('js/app.js');

let reader = new FileReaderSync();

onmessage = async function(e) {
  let file = e.data;
  let wasm = await Rust.asstosrt_wasm;
  console.log("convert " + file.name);

  try {
    let ass = reader.readAsText(file);
    let srt = wasm.assToSrt(ass);
    let srtFile = new Blob([srt], {type: 'text/srt'});
    let srtUrl = URL.createObjectURL(srtFile);
    postMessage({file: file, srtUrl: srtUrl});
  } catch (e) {
    postMessage({file: file, error: e});
  }
}
