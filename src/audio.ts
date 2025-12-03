import { invoke } from "@tauri-apps/api/core";

let audioCtx: AudioContext | null = null;
let mediaStream: MediaStream | null = null;
let workletNode: AudioWorkletNode | null = null;

export async function startStream() {
  if (audioCtx) return;

  try {
    mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
    audioCtx = new AudioContext({ sampleRate: 16000 });
    await audioCtx.audioWorklet.addModule("pcm-processor.js"); // adjust path

    const source = audioCtx.createMediaStreamSource(mediaStream);
    workletNode = new AudioWorkletNode(audioCtx, "pcm-processor");

    workletNode.port.onmessage = (ev) => {
      const chunk: Float32Array = ev.data;
      // convert Float32 [-1..1] -> Int16LE -> Uint8Array
      const pcm16 = new Int16Array(chunk.length);
      for (let i = 0; i < chunk.length; i++) {
        let s = Math.max(-1, Math.min(1, chunk[i]));
        pcm16[i] = s < 0 ? s * 0x8000 : s * 0x7fff;
      }
      const u8 = new Uint8Array(pcm16.buffer);
      invoke("push_pcm_chunk", { chunk: Array.from(u8) }).catch(console.error);
    };

    source.connect(workletNode);
    workletNode.connect(audioCtx.destination);

    console.log("Audio stream started with AudioWorklet.");
  } catch (err) {
    console.error("Failed to start audio stream:", err);
  }
}

export function stopStream() {
  if (!audioCtx) return;

  workletNode?.disconnect();
  mediaStream?.getTracks().forEach((t) => t.stop());
  audioCtx.close();

  audioCtx = null;
  mediaStream = null;
  workletNode = null;

  console.log("Audio stream stopped.");
}
