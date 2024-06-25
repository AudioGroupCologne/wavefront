import pyfar as pf
import numpy as np
import matplotlib.pyplot as plt
import pandas as pd

df = pd.read_csv("scripts/mic_0.csv")
df = df.drop(df.columns[0], axis=1)

data = df.to_numpy().reshape(-1)

signal = pf.Signal(data, 48000)

ax = pf.plot.freq(signal, label="Signal in dB")
ax.legend(loc='upper left')
plt.show()