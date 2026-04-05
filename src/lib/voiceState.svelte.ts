let _ttsEnabled = $state(false);
let _isRecording = $state(false);
let _isTranscribing = $state(false);
let _isSpeaking = $state(false);
let _modelsReady = $state(false);

export const voiceState = {
  get ttsEnabled() { return _ttsEnabled; },
  set ttsEnabled(v: boolean) { _ttsEnabled = v; },

  get isRecording() { return _isRecording; },
  set isRecording(v: boolean) { _isRecording = v; },

  get isTranscribing() { return _isTranscribing; },
  set isTranscribing(v: boolean) { _isTranscribing = v; },

  get isSpeaking() { return _isSpeaking; },
  set isSpeaking(v: boolean) { _isSpeaking = v; },

  get modelsReady() { return _modelsReady; },
  set modelsReady(v: boolean) { _modelsReady = v; },
};
