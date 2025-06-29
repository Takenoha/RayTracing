import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.patches import Circle, Rectangle, Arc, Polygon
import numpy as np
import toml
import glob

def get_rotated_rect_corners(center, size, angle_deg):
    """Y軸周りに回転した長方形の4つの角の座標を計算する"""
    cx, cz = center
    w, h = size
    angle_rad = np.radians(angle_deg)
    
    # 長方形のローカル座標での4隅
    local_corners = np.array([
        [-w/2, -h/2], [w/2, -h/2], [w/2, h/2], [-w/2, h/2]
    ])
    
    # 回転行列
    rotation_matrix = np.array([
        [np.cos(angle_rad), -np.sin(angle_rad)],
        [np.sin(angle_rad),  np.cos(angle_rad)]
    ])
    
    # 回転させてから移動
    world_corners = local_corners @ rotation_matrix.T + np.array([cx, cz])
    return world_corners

# --- 1. ファイル読み込み ---
try:
    with open('scene.toml', 'r', encoding='utf-8') as f:
        scene_config = toml.load(f)
except FileNotFoundError:
    print("エラー: 'scene.toml' が見つかりません。")
    exit()

path_files = sorted(glob.glob('path_*.csv'))
if not path_files:
    print("エラー: path_*.csv が見つかりません。先にRustプログラムを実行してください。")
    exit()

# --- 2. プロット準備 ---
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(22, 11))
ax1.set_title('Top View (XZ Projection)')
ax1.set_xlabel('X coordinate')
ax1.set_ylabel('Z coordinate')
ax1.grid(True)
ax1.set_facecolor('ivory')

ax2.set_title('Side View (YZ Projection)')
ax2.set_xlabel('Z coordinate')
ax2.set_ylabel('Y coordinate')
ax2.grid(True)
ax2.set_facecolor('ivory')


# --- 3. TOMLデータに基づいてオブジェクトを描画 ---
for obj in scene_config.get('objects', []):
    pos = obj['transform']['position']
    rot_y = obj['transform']['rotation_y_deg']
    shape_cfg = obj['shape']
    shape_type = shape_cfg['type']
    
    obj_color = 'deepskyblue' if 'Glass' in obj['material'] else 'dimgray'

    # --- 上から見た図 (XZ平面) ---
    if shape_type in ['Sphere', 'Cylinder', 'Lens']:
        radius = shape_cfg.get('radius', shape_cfg.get('diameter', 0) / 2.0)
        patch = Circle((pos[0], pos[2]), radius, ec='black', fc=obj_color, alpha=0.4)
        ax1.add_patch(patch)
    elif shape_type == 'Box':
        size = (shape_cfg['size'][0], shape_cfg['size'][2])
        corners = get_rotated_rect_corners((pos[0], pos[2]), size, rot_y)
        patch = Polygon(corners, ec='black', fc=obj_color, alpha=0.4)
        ax1.add_patch(patch)
    elif shape_type == 'Wedge':
        size = (shape_cfg['size'][0], shape_cfg['size'][2])
        corners = get_rotated_rect_corners((pos[0], pos[2]), size, rot_y)
        # 簡単化のため、ウェッジの上面図は外接する長方形として描画
        patch = Polygon(corners, ec='black', fc=obj_color, alpha=0.4)
        ax1.add_patch(patch)


    # --- 横から見た図 (YZ平面) ---
    if shape_type == 'Sphere':
        patch = Circle((pos[2], pos[1]), shape_cfg['radius'], ec='black', fc=obj_color, alpha=0.4)
        ax2.add_patch(patch)
    elif shape_type == 'Box':
        size = (shape_cfg['size'][2], shape_cfg['size'][1])
        patch = Rectangle((pos[2] - size[0]/2, pos[1] - size[1]/2), size[0], size[1], ec='black', fc=obj_color, alpha=0.4)
        ax2.add_patch(patch)
    elif shape_type == 'Cylinder':
        size = (shape_cfg['radius']*2, shape_cfg['height'])
        # YZ平面では円柱は常に長方形に見える（Y軸回転は影響しない）
        patch = Rectangle((pos[2] - shape_cfg['radius'], pos[1] - size[1]/2), size[0], size[1], ec='black', fc=obj_color, alpha=0.4)
        ax2.add_patch(patch)
    elif shape_type == 'Lens':
        # レンズの円弧描画ロジック（以前のものと同様）
        # ... この部分は必要に応じて詳細化 ...
        pass
    elif shape_type == 'Wedge':
        size = shape_cfg['size']
        angle_rad = np.radians(shape_cfg['angle_deg'])
        # ウェッジの側面図は三角形
        p1 = (pos[2] - size[2]/2, pos[1])
        p2 = (pos[2] + size[2]/2, pos[1])
        p3_x_local = size[2]/2 - size[1] / np.tan(angle_rad) # XZ平面でのx座標
        p3 = (pos[2] + p3_x_local, pos[1] + size[1])
        patch = Polygon([p1, p2, p3], ec='black', fc=obj_color, alpha=0.4)
        ax2.add_patch(patch)
        
# --- 4. 光路の描画 ---
for i, file_path in enumerate(path_files):
    df = pd.read_csv(file_path)
    label = 'Ray Path' if i == 0 else None
    
    # 上面図 (XZ平面)
    ax1.plot(df['x'], df['z'], linestyle='-', color='red', alpha=0.9, lw=1.5, label=label)
    ax1.plot(df['x'].iloc[0], df['z'].iloc[0], 'go', markersize=8)
    
    # 側面図 (YZ平面)
    ax2.plot(df['z'], df['y'], linestyle='-', color='red', alpha=0.9, lw=1.5, label=label)
    ax2.plot(df['z'].iloc[0], df['y'].iloc[0], 'go', markersize=8)

# --- 5. グラフの体裁 ---
ax1.legend()
ax1.set_aspect('equal', adjustable='box')
ax2.legend()
ax2.set_aspect('equal', adjustable='box')

plt.tight_layout()
plt.show()