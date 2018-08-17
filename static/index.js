let $ = s => document.querySelector(s);
let worker = new Worker("worker.js");
let nextId = 1;

$("#files").addEventListener("change", async ev => {
  let files = [];
  for (let i = 0; i < ev.target.files.length; i++) {
    files.push(ev.target.files.item(i));
  }
  await addFiles(files);
});

const preloadDict = (d) => worker.postMessage({
  action: "preloadDict", dict: d,
});
preloadDict($("#conv-dict").value);
$("#conv-dict").addEventListener("change", ev =>
  preloadDict(ev.target.value));

async function onDrop(ev) {
  ev.preventDefault();
  let items = ev.dataTransfer.items
    .filter(i => i.kind == "file")
    .map(f => f.getAsFile());
  await addFiles(items);
};

function onDropOver(ev) {
  ev.preventDefault();
};

function onDropEnd(ev) {
  ev.dataTransfer.clearData();
}

async function addFiles(files) {
  let id = nextId++;
  let template = document.querySelector("#file");
  let content = document.importNode(template, true).content;
  content.querySelector(".file").id = `file-${id}`;
  if (files.length == 1) {
    content.querySelector(".name").textContent = files[0].name;
    content.querySelector(".save").download = `${files[0].name}.srt`;
  } else {
    content.querySelector(".name").textContent =
      `${files.length} subtitle files`;
    content.querySelector(".save").download = `srt_subtitles.zip`;
  }
  content.querySelector(".close").addEventListener("click", event => {
    event.preventDefault();
    $(`#file-${id}`).outerHTML = "";
  });
  $("#list").appendChild(content);

  let size = files.reduce((n, f) => n + f.size, 0);
  if (size > 100 * 1024 * 1024)
    return onConvertError(id, "files too large (> 100 MiB)");

  let opts = {
    in_charset: $("#in-charset").value || null,
    out_charset: $("#out-charset").value || null,
    lines: $("#lines").value,
    ignore_codec_err: $("#ignore-codec-err").checked,
  };
  let cmd = { id: id, opts: opts };
  if (files.length == 1) {
    cmd.action = "addFile";
    cmd.file = files[0];
  } else {
    cmd.action = "addFiles";
    cmd.files = files;
  }
  worker.postMessage(cmd);
}

function onConvertError(id, msg) {
  let content = $(`#file-${id}`);
  content.classList.remove("progress");
  content.classList.add("error");
  content.querySelector(".status").textContent = msg;
}

function onConvertDone(id, url) {
  let content = $(`#file-${id}`);
  content.classList.remove("progress");
  content.classList.add("done");
  content.querySelector(".save").href = url;
  content.querySelector(".close").addEventListener("click", event => {
    URL.revokeObjectURL(url);
  });
}

worker.onmessage = function(e) {
  let result = e.data;
  if (result.error) {
    onConvertError(result.id, result.error);
  } else {
    onConvertDone(result.id, result.url);
  }
}
