let audioCtxCapture: AudioContext | null = null;
let sourceNode: MediaStreamAudioSourceNode | null = null;
let scriptNode: ScriptProcessorNode | null = null;
let capturedSamples: Float32Array[] = [];
let mediaStream: MediaStream | null = null;

export async function startRecording(): Promise<void> {
  mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });

  // Capture raw PCM at 16kHz directly — avoids codec issues in WebKitGTK
  audioCtxCapture = new AudioContext({ sampleRate: 16000 });
  sourceNode = audioCtxCapture.createMediaStreamSource(mediaStream);
  // 4096 buffer size, mono input, mono output
  scriptNode = audioCtxCapture.createScriptProcessor(4096, 1, 1);
  capturedSamples = [];

  scriptNode.onaudioprocess = (e) => {
    const input = e.inputBuffer.getChannelData(0);
    capturedSamples.push(new Float32Array(input));
  };

  sourceNode.connect(scriptNode);
  scriptNode.connect(audioCtxCapture.destination);
}

export async function stopRecording(): Promise<Uint8Array> {
  if (!audioCtxCapture || !scriptNode || !sourceNode || !mediaStream) {
    throw new Error('No recording in progress');
  }

  // Disconnect and clean up
  scriptNode.disconnect();
  sourceNode.disconnect();
  mediaStream.getTracks().forEach((t) => t.stop());
  await audioCtxCapture.close();

  audioCtxCapture = null;
  sourceNode = null;
  scriptNode = null;
  mediaStream = null;

  // Concatenate all captured chunks into a single Float32Array
  const totalLength = capturedSamples.reduce((sum, chunk) => sum + chunk.length, 0);
  const allSamples = new Float32Array(totalLength);
  let offset = 0;
  for (const chunk of capturedSamples) {
    allSamples.set(chunk, offset);
    offset += chunk.length;
  }
  capturedSamples = [];

  return float32ToBytes(allSamples);
}

export function float32ToBytes(samples: Float32Array): Uint8Array {
  const buffer = new ArrayBuffer(samples.length * 4);
  const view = new DataView(buffer);
  for (let i = 0; i < samples.length; i++) {
    view.setFloat32(i * 4, samples[i], true); // little-endian
  }
  return new Uint8Array(buffer);
}

let currentAudioSource: AudioBufferSourceNode | null = null;
let currentAudioCtx: AudioContext | null = null;

export async function playWavBytes(wavBytes: Uint8Array): Promise<void> {
  stopPlayback();

  const audioCtx = new AudioContext();
  currentAudioCtx = audioCtx;

  const arrayBuffer = wavBytes.buffer.slice(
    wavBytes.byteOffset,
    wavBytes.byteOffset + wavBytes.byteLength
  );
  const audioBuffer = await audioCtx.decodeAudioData(arrayBuffer);

  const source = audioCtx.createBufferSource();
  currentAudioSource = source;
  source.buffer = audioBuffer;
  source.connect(audioCtx.destination);

  return new Promise((resolve) => {
    source.onended = () => {
      currentAudioSource = null;
      currentAudioCtx = null;
      audioCtx.close();
      resolve();
    };
    source.start();
  });
}

export function stopPlayback(): void {
  if (currentAudioSource) {
    try {
      currentAudioSource.stop();
    } catch {
      // Already stopped
    }
    currentAudioSource = null;
  }
  if (currentAudioCtx) {
    currentAudioCtx.close();
    currentAudioCtx = null;
  }
}

