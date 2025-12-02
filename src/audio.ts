import { invoke } from "@tauri-apps/api/core";

export function startMicStream() {
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    console.error("getUserMedia not supported");
    return;
  }

  navigator.mediaDevices.getUserMedia({ audio: true }).then((stream) => {
    const mediaRecorder = new MediaRecorder(stream, { mimeType: "audio/webm; codecs=pcm" });

    mediaRecorder.ondataavailable = (event) => {
  if (event.data && event.data.size > 0) {
    event.data.arrayBuffer().then((buffer) => {
      invoke("mic_chunk", { chunk: Array.from(new Uint8Array(buffer)) });
    });
  }
};

    mediaRecorder.start(100); // emit chunks every 100ms
  });
}
