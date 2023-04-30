from gtts import gTTS
import os

# define variables
file = "file.mp3"

# initialize tts, create mp3 and play
tts = gTTS(s, 'com')
tts.save(file)
