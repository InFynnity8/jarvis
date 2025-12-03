# tts_helper.py
import sys

from pathlib import Path
from TTS.api import TTS

output_path = Path("src/models/output.wav")

# Ensure parent directory exists
output_path.parent.mkdir(parents=True, exist_ok=True)

tts = TTS(model_name="tts_models/en/ljspeech/tacotron2-DDC", progress_bar=False)

def text_to_wav(text, out_path):
    tts.tts_to_file(text=text, file_path=out_path)

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python tts.py <text> <output_wav>")
        sys.exit(1)
    text = sys.argv[1]
    output_file = sys.argv[2]
    text_to_wav(text, output_file)
