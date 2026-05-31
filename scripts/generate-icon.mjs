import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

const root = process.cwd();
const iconPath = path.join(root, 'apps/desktop/src-tauri/icons/icon.png');
const publicIconPath = path.join(root, 'apps/desktop/public/icon.png');

const size = 512;
const data = Buffer.alloc((size * 4 + 1) * size);

for (let y = 0; y < size; y += 1) {
  const row = y * (size * 4 + 1);
  data[row] = 0;
  for (let x = 0; x < size; x += 1) {
    const idx = row + 1 + x * 4;
    const t = y / size;
    const base = mix([10, 72, 88], [15, 118, 110], t);
    data[idx] = base[0];
    data[idx + 1] = base[1];
    data[idx + 2] = base[2];
    data[idx + 3] = 255;
  }
}

drawRoundedRect(88, 72, 336, 368, 54, [248, 250, 252, 255]);
drawRoundedRect(122, 126, 268, 244, 26, [15, 118, 110, 255]);
drawRect(122, 176, 268, 14, [248, 250, 252, 255]);
drawRect(180, 126, 16, 64, [248, 250, 252, 255]);
drawRect(314, 126, 16, 64, [248, 250, 252, 255]);
drawCalendarCells();
drawShieldCheck();

const png = Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk('IHDR', header(size, size)),
  chunk('IDAT', zlib.deflateSync(data)),
  chunk('IEND', Buffer.alloc(0)),
]);

fs.mkdirSync(path.dirname(iconPath), { recursive: true });
fs.mkdirSync(path.dirname(publicIconPath), { recursive: true });
fs.writeFileSync(iconPath, png);
fs.writeFileSync(publicIconPath, png);

function drawCalendarCells() {
  const color = [248, 250, 252, 255];
  for (const y of [224, 278, 332]) {
    for (const x of [156, 224, 292]) {
      drawRoundedRect(x, y, 36, 28, 6, color);
    }
  }
  drawRoundedRect(292, 332, 36, 28, 6, [250, 204, 21, 255]);
}

function drawShieldCheck() {
  const points = [
    [256, 300],
    [332, 324],
    [316, 394],
    [256, 432],
    [196, 394],
    [180, 324],
  ];
  fillPolygon(points, [37, 99, 235, 255]);
  drawThickLine(222, 366, 250, 394, 14, [248, 250, 252, 255]);
  drawThickLine(250, 394, 300, 342, 14, [248, 250, 252, 255]);
}

function drawRoundedRect(x, y, width, height, radius, color) {
  for (let yy = y; yy < y + height; yy += 1) {
    for (let xx = x; xx < x + width; xx += 1) {
      const dx = xx < x + radius ? x + radius - xx : xx >= x + width - radius ? xx - (x + width - radius - 1) : 0;
      const dy = yy < y + radius ? y + radius - yy : yy >= y + height - radius ? yy - (y + height - radius - 1) : 0;
      if (dx * dx + dy * dy <= radius * radius) {
        setPixel(xx, yy, color);
      }
    }
  }
}

function drawRect(x, y, width, height, color) {
  for (let yy = y; yy < y + height; yy += 1) {
    for (let xx = x; xx < x + width; xx += 1) {
      setPixel(xx, yy, color);
    }
  }
}

function drawThickLine(x0, y0, x1, y1, radius, color) {
  const minX = Math.min(x0, x1) - radius;
  const maxX = Math.max(x0, x1) + radius;
  const minY = Math.min(y0, y1) - radius;
  const maxY = Math.max(y0, y1) + radius;
  const dx = x1 - x0;
  const dy = y1 - y0;
  const lenSq = dx * dx + dy * dy;
  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const t = Math.max(0, Math.min(1, ((x - x0) * dx + (y - y0) * dy) / lenSq));
      const px = x0 + t * dx;
      const py = y0 + t * dy;
      if ((x - px) ** 2 + (y - py) ** 2 <= radius ** 2) {
        setPixel(x, y, color);
      }
    }
  }
}

function fillPolygon(points, color) {
  const minY = Math.floor(Math.min(...points.map((point) => point[1])));
  const maxY = Math.ceil(Math.max(...points.map((point) => point[1])));
  for (let y = minY; y <= maxY; y += 1) {
    const intersections = [];
    for (let i = 0; i < points.length; i += 1) {
      const [x1, y1] = points[i];
      const [x2, y2] = points[(i + 1) % points.length];
      if ((y1 <= y && y2 > y) || (y2 <= y && y1 > y)) {
        intersections.push(x1 + ((y - y1) * (x2 - x1)) / (y2 - y1));
      }
    }
    intersections.sort((a, b) => a - b);
    for (let i = 0; i < intersections.length; i += 2) {
      for (let x = Math.ceil(intersections[i]); x <= Math.floor(intersections[i + 1]); x += 1) {
        setPixel(x, y, color);
      }
    }
  }
}

function setPixel(x, y, color) {
  if (x < 0 || x >= size || y < 0 || y >= size) {
    return;
  }
  const idx = y * (size * 4 + 1) + 1 + x * 4;
  data[idx] = color[0];
  data[idx + 1] = color[1];
  data[idx + 2] = color[2];
  data[idx + 3] = color[3];
}

function mix(a, b, t) {
  return a.map((value, index) => Math.round(value + (b[index] - value) * t));
}

function header(width, height) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  return ihdr;
}

function chunk(type, payload) {
  const typeBuffer = Buffer.from(type);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(payload.length);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, payload])));
  return Buffer.concat([length, typeBuffer, payload, crc]);
}

function crc32(buffer) {
  let crc = ~0;
  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
    }
  }
  return ~crc >>> 0;
}
