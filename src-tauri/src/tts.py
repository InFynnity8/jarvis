# tts_helper.py
import sys
import wave
from TTS.api import TTS

# Load model once
tts = TTS(model_name="models/en-us", progress_bar=False, gpu=False)

def text_to_wav(text, out_path):
    tts.tts_to_file(text=text, file_path=out_path)

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python tts.py <text> <output_wav>")
        sys.exit(1)
    text = sys.argv[1]
    output_file = sys.argv[2]
    text_to_wav(text, output_file)
