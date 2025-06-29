import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.patches import Circle, PathPatch
from matplotlib.path import Path

# CSVファイルを読み込む
try:
    df = pd.read_csv('path_3d.csv')
except FileNotFoundError:
    print("Error: 'path_3d.csv' not found. Please run the Rust program first.")
    exit()

# プロットの準備
plt.figure(figsize=(12, 9))
ax = plt.gca()

# --- シーン内のオブジェクトの断面を描画 ---
# Rustコードのシーン設定と対応させる

# 1. ガラスの球: center=(0,0,0), radius=10
#    XY平面で見た場合、(0,0)が中心の半径10の円になる
sphere = Circle((0, 0), 10, color='skyblue', alpha=0.4, label='Glass Sphere (ior=1.5)')
ax.add_patch(sphere)

# 2. 反射する床: y = -15 の平面
#    XY平面で見た場合、y = -15 の水平な直線になる
plt.axhline(y=-15, color='dimgray', linewidth=3, label='Mirror Plane')


# --- 光路を描画 ---
# 'x'と'y'の列だけを使ってプロット（Z軸を無視してXY平面に射影）
plt.plot(df['x'], df['z'], marker='o', markersize=4, linestyle='-', color='red', label='Ray Path (XZ Projection)')

# 始点に印をつける
plt.plot(df['x'].iloc[0], df['z'].iloc[0], 'go', markersize=10, label='Start Point')

# --- グラフの体裁を整える ---
plt.xlabel('X coordinate')
plt.ylabel('Y coordinate')
plt.title('3D Optical Path Simulation (Projected onto XY Plane)')
plt.grid(True)
plt.legend()
plt.axis('equal') # X軸とY軸のスケールを合わせる（形状が歪まないように）
plt.show()