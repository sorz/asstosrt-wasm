window = 'fix stdweb';
importScripts('js/app.js');

let reader = new FileReaderSync();

onmessage = async function(e) {
  let id = e.data.id;
  let file = e.data.file;
  let opts = e.data.opts;
  try {
    let wasm = await Rust.asstosrt_wasm;
    let dict = await fetchChineseConvDict(e.data.dict);
    let ass = reader.readAsArrayBuffer(file);
    let srt = wasm.assToSrt(ass, opts, dict);
    let srtUrl = URL.createObjectURL(srt);
    postMessage({id: id, srtUrl: srtUrl});
  } catch (e) {
    postMessage({id: id, error: e});
  }
}

async function fetchChineseConvDict(dict) {
  if (!dict) return null;
  let resp = await fetch(dict);
  if (!resp.ok) throw "fail to download dict: " + resp.status;
  return await resp.arrayBuffer();
}

