window = 'fix stdweb';
importScripts('js/app.js');

let reader = new FileReaderSync();

onmessage = async function(e) {
  let id = e.data.id;
  let opts = e.data;

  let wasm = await Rust.asstosrt_wasm;
  console.log("convert " + opts.file.name);

  try {
    let srt = wasm.assToSrt(
      reader.readAsArrayBuffer(opts.file),
      opts.inCharset,
      opts.outCharset,
      opts.chineseConv
    );
    console.log(srt);
    let srtUrl = URL.createObjectURL(srt);
    postMessage({id: id, srtUrl: srtUrl});
  } catch (e) {
    postMessage({id: id, error: e});
  }
}
