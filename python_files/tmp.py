import matplotlib.pyplot as plt

# 提示された座標データ
points = [
    (-10.000, 2.000),
    (-4.583, 2.000),
    (4.959, 0.640),
    (24.163, -4.948)
]

# x座標とy座標をそれぞれのリストに分割
# zip(*points)は、[(x1, y1), (x2, y2)] を ([x1, x2], [y1, y2]) に変換するテクニックです
x_coords, y_coords = zip(*points)

# グラフの準備
plt.figure(figsize=(10, 8))

# x, y座標をプロットし、点と線を両方描画する
# marker='o' で各点に円を描画
# linestyle='-' で点を線で結ぶ
plt.plot(x_coords, y_coords, marker='o', linestyle='-', label='Ray Path (XY Projection)')

# 始点に緑のマーカーを付ける
plt.plot(x_coords[0], y_coords[0], 'go', markersize=10, label='Start Point')

# グラフの体裁を整える
plt.title('光路のXY座標プロット (XY Coordinate Plot of the Light Path)')
plt.xlabel('X coordinate')
plt.ylabel('Y coordinate')
plt.grid(True)
plt.legend()
plt.axis('equal')  # X軸とY軸のスケールを等しくして、角度が正しく表示されるようにする

# グラフを表示
plt.show()