// pcm-processor.ts
class PCMProcessor extends AudioWorkletProcessor {
  buffer = [];
  CHUNK_SIZE = 16000; // 1 second @ 16kHz

  constructor() {
    super();
  }

  process(inputs) {
    const input = inputs[0];
    if (input.length === 0) return true;

    const channelData = input[0]; // mono
    this.buffer.push(channelData.slice());

    // combine enough samples and send to main thread
    let totalLen = this.buffer.reduce((acc, b) => acc + b.length, 0);
    if (totalLen >= this.CHUNK_SIZE) {
      const chunk = new Float32Array(this.CHUNK_SIZE);
      let offset = 0;
      while (offset < this.CHUNK_SIZE && this.buffer.length > 0) {
        const b = this.buffer[0];
        const take = Math.min(b.length, this.CHUNK_SIZE - offset);
        chunk.set(b.slice(0, take), offset);
        offset += take;
        if (take < b.length) this.buffer[0] = b.slice(take);
        else this.buffer.shift();
      }
      this.port.postMessage(chunk);
    }

    return true; // keep processor alive
  }
}

registerProcessor("pcm-processor", PCMProcessor);
