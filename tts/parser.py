import multiprocessing
from playsound import playsound
import threading
import subprocess
import json
from gtts import gTTS
import os

def read_output(proc):
    for line in iter(proc.stdout.readline, b''):
        data = json.loads(line.decode('utf-8'))
        print(data)
        if "u" in data and "qtts" in data["u"] and len(data["u"]["qtts"]) > 0:
            qd = data["u"]["qtts"]["questDescription"]
            print(data["u"]["qtts"]["questDescription"])
            tts = gTTS(qd, 'com')
            tts.save("out.mp3")
            p = multiprocessing.Process(target=playsound, args=("out.mp3",))
            p.start()

if __name__ == '__main__':
    proc = subprocess.Popen(['C:\\Users\\alexb\\Code\\LibSerpix\\ScreenReaderDemo\\target\\release\\wow.exe'], stdout=subprocess.PIPE)
    t = threading.Thread(target=read_output, args=(proc,))
    t.start()
