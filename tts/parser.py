from gtts import gTTS
from playsound import playsound
import json
import multiprocessing
import os
import subprocess
import threading

def read_output(proc):
    for line in iter(proc.stdout.readline, b''):
        data = json.loads(line.decode('utf-8'))
        if "u" in data and "qtts" in data["u"] and len(data["u"]["qtts"]) > 0:
            qd = ""
            for k in data["u"]["qtts"]:
                qd += data["u"]["qtts"][k]
            tts = gTTS(qd, 'com')
            tts.save("out.mp3")
            p = multiprocessing.Process(target=playsound, args=("out.mp3",))
            p.start()

if __name__ == '__main__':
    proc = None
    print('C:\\Users\\%s\\Code\\LibSerpix\\ScreenReaderDemo\\target\\release\\wow.exe'%os.getenv('USERNAME'))
    if os.path.isfile("wow.exe"):
        proc = subprocess.Popen(['wow.exe'], stdout=subprocess.PIPE)
    else:
        proc = subprocess.Popen(['C:\\Users\\%s\\Code\\LibSerpix\\ScreenReaderDemo\\target\\release\\wow.exe'%os.getenv('USERNAME')
], stdout=subprocess.PIPE)
    t = threading.Thread(target=read_output, args=(proc,))
    t.start()
