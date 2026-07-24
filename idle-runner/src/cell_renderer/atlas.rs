// SPDX-License-Identifier: MIT

use super::CellRenderer;

impl CellRenderer {
    pub(crate) fn prepopulate_atlas(&mut self) {
        // ASCII
        for ch in 32..=126 {
            if let Some(c) = char::from_u32(ch) {
                self.get_or_insert_atlas_char(c);
            }
        }
        // Katakana
        let katakana = "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ1234567890X:+-*<>|";
        for c in katakana.chars() {
            self.get_or_insert_atlas_char(c);
        }
        // Special screensaver symbols
        let symbols = &['✦', '·', '░', '╬', '█', '▲', '∩', '¥', '✹'];
        for &c in symbols {
            self.get_or_insert_atlas_char(c);
        }
        self.rebuild_atlas_image();
    }

    pub fn get_or_insert_atlas_char(&mut self, ch: char) -> usize {
        if let Some(pos) = self.atlas_chars.iter().position(|&c| c == ch) {
            pos
        } else {
            self.atlas_chars.push(ch);
            self.atlas_dirty = true;
            self.atlas_chars.len() - 1
        }
    }

    pub fn rebuild_atlas_image(&mut self) {
        if !self.atlas_dirty && !self.atlas_image.is_empty() {
            return;
        }

        let needed_cells = self.atlas_chars.len();
        while needed_cells > self.atlas_cols * self.atlas_rows {
            self.atlas_rows *= 2;
        }

        let atlas_w = self.atlas_cols * self.cell_width;
        let atlas_h = self.atlas_rows * self.cell_height;
        self.atlas_image.resize(atlas_w * atlas_h, 0);
        self.atlas_image.fill(0);

        let chars = self.atlas_chars.clone();
        for (idx, ch) in chars.into_iter().enumerate() {
            let (metrics, bitmap) = self.glyph_for(ch);
            let col = idx % self.atlas_cols;
            let row = idx / self.atlas_cols;

            let char_x = col * self.cell_width;
            let char_y = row * self.cell_height;

            let y_offset = metrics.ymin.max(0) as usize;
            for r in 0..metrics.height {
                let dst_y = char_y + y_offset + r;
                if dst_y >= char_y + self.cell_height {
                    continue;
                }
                for c in 0..metrics.width {
                    let dst_x = char_x + c;
                    if dst_x >= char_x + self.cell_width {
                        continue;
                    }
                    let src_idx = r * metrics.width + c;
                    let dst_idx = dst_y * atlas_w + dst_x;
                    if src_idx < bitmap.len() && dst_idx < self.atlas_image.len() {
                        self.atlas_image[dst_idx] = bitmap[src_idx];
                    }
                }
            }
        }
        self.atlas_dirty = false;
    }

    pub fn atlas_info(&mut self) -> (usize, usize, usize, usize, &[u8], bool) {
        if self.atlas_dirty {
            self.rebuild_atlas_image();
        }
        (
            self.atlas_cols * self.cell_width,
            self.atlas_rows * self.cell_height,
            self.atlas_cols,
            self.atlas_rows,
            &self.atlas_image,
            self.atlas_dirty,
        )
    }

    pub fn reset_atlas_dirty(&mut self) {
        self.atlas_dirty = false;
    }

    pub fn is_atlas_dirty(&self) -> bool {
        self.atlas_dirty
    }
}
