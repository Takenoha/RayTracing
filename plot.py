import matplotlib.pyplot as plt
import pandas as pd

df = pd.read_csv('path_output.csv')

plt.figure(figsize=(10, 8))
# 光路を描画
plt.plot(df['x'], df['y'], marker='o', label='Ray Path')

# オブジェクトを描画（手動）
lens = plt.Circle((20, 0), 15, color='lightblue', alpha=0.5, label='Lens')
mirror = plt.Line2D([30, 30], [-20, 20], color='gray', linewidth=3, label='Mirror')

ax = plt.gca()
ax.add_patch(lens)
ax.add_line(mirror)

plt.xlabel('X')
plt.ylabel('Y')
plt.title('2D Optical Path Simulation')
plt.grid(True)
plt.legend()
plt.axis('equal')
plt.show()