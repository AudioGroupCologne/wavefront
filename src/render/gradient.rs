use bevy::ecs::system::Resource;
use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::math::transformations::map_range;

#[derive(Resource, Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum Gradient {
    #[default]
    Turbo,
    Viridis,
    Magma,
    Plasma,
    Inferno,
    Bw,
}

impl Gradient {
    /// Returns the color at a certain percentage (between 0.0..=1.0) of the gradient
    pub fn at(&self, percent: f32, min: f32, max: f32) -> [u8; 3] {
        let idx = map_range(min, max, 0., 255., percent) as u8 as usize;
        let map = match self {
            Gradient::Turbo => &TURBO,
            Gradient::Magma => &MAGMA,
            Gradient::Viridis => &VIRIDIS,
            Gradient::Plasma => &PLASMA,
            Gradient::Inferno => &INFERNO,
            Gradient::Bw => &BW,
        };

        map[idx]
    }
}

// https://gist.github.com/mikhailov-work/6a308c20e494d9e0ccc29036b28faa7a
const TURBO: [[u8; 3]; 256] = [
    [6, 0, 10],
    [6, 1, 13],
    [7, 1, 16],
    [7, 1, 20],
    [8, 2, 24],
    [8, 2, 29],
    [8, 3, 33],
    [9, 3, 38],
    [9, 4, 43],
    [9, 5, 49],
    [10, 6, 55],
    [10, 7, 60],
    [10, 8, 67],
    [10, 8, 73],
    [11, 9, 79],
    [11, 10, 86],
    [12, 12, 92],
    [12, 13, 99],
    [12, 14, 105],
    [12, 15, 112],
    [13, 17, 119],
    [13, 18, 127],
    [13, 19, 133],
    [13, 21, 139],
    [13, 23, 147],
    [13, 24, 154],
    [14, 26, 159],
    [14, 28, 166],
    [14, 29, 173],
    [14, 31, 178],
    [14, 33, 186],
    [14, 35, 191],
    [14, 37, 197],
    [14, 39, 203],
    [14, 41, 207],
    [14, 44, 213],
    [14, 45, 217],
    [14, 48, 223],
    [14, 50, 227],
    [14, 53, 231],
    [14, 55, 235],
    [14, 57, 239],
    [14, 59, 241],
    [14, 62, 246],
    [14, 65, 248],
    [13, 68, 250],
    [13, 70, 250],
    [13, 73, 252],
    [12, 75, 252],
    [12, 79, 252],
    [11, 81, 252],
    [11, 85, 252],
    [10, 87, 250],
    [10, 91, 248],
    [9, 93, 248],
    [9, 97, 246],
    [8, 101, 241],
    [7, 104, 239],
    [7, 107, 235],
    [6, 111, 233],
    [6, 114, 229],
    [5, 118, 225],
    [5, 121, 221],
    [4, 125, 217],
    [4, 128, 213],
    [3, 131, 209],
    [3, 136, 203],
    [3, 139, 199],
    [2, 142, 193],
    [2, 146, 189],
    [2, 151, 184],
    [2, 154, 180],
    [1, 157, 175],
    [1, 161, 169],
    [1, 164, 166],
    [1, 168, 161],
    [1, 171, 156],
    [1, 175, 152],
    [1, 178, 147],
    [1, 180, 142],
    [1, 184, 139],
    [1, 187, 135],
    [1, 191, 131],
    [1, 193, 127],
    [1, 197, 124],
    [1, 199, 121],
    [1, 201, 118],
    [2, 205, 114],
    [2, 207, 111],
    [2, 209, 107],
    [3, 213, 103],
    [3, 215, 99],
    [4, 217, 95],
    [4, 219, 91],
    [5, 221, 87],
    [6, 223, 84],
    [7, 225, 80],
    [8, 229, 77],
    [9, 231, 73],
    [10, 231, 69],
    [11, 233, 66],
    [13, 235, 62],
    [14, 237, 58],
    [16, 239, 55],
    [18, 241, 52],
    [20, 241, 49],
    [22, 244, 46],
    [25, 246, 43],
    [27, 246, 40],
    [30, 248, 38],
    [33, 248, 35],
    [36, 250, 33],
    [39, 250, 31],
    [42, 250, 29],
    [45, 252, 27],
    [48, 252, 25],
    [52, 252, 23],
    [55, 252, 21],
    [59, 252, 19],
    [62, 252, 18],
    [67, 252, 17],
    [70, 252, 15],
    [74, 252, 14],
    [78, 252, 13],
    [81, 252, 13],
    [85, 250, 12],
    [88, 250, 11],
    [92, 248, 10],
    [96, 248, 10],
    [99, 246, 9],
    [103, 246, 9],
    [107, 244, 8],
    [109, 241, 8],
    [114, 239, 8],
    [117, 239, 8],
    [121, 237, 8],
    [125, 233, 7],
    [128, 231, 7],
    [133, 229, 7],
    [136, 227, 7],
    [141, 225, 7],
    [144, 221, 7],
    [149, 219, 7],
    [152, 217, 7],
    [157, 213, 7],
    [161, 211, 7],
    [164, 207, 7],
    [169, 205, 8],
    [173, 201, 8],
    [176, 197, 8],
    [180, 195, 8],
    [186, 191, 8],
    [189, 187, 8],
    [193, 184, 8],
    [197, 180, 8],
    [201, 176, 9],
    [205, 175, 9],
    [207, 171, 9],
    [211, 168, 9],
    [215, 164, 9],
    [217, 161, 9],
    [221, 157, 9],
    [223, 154, 9],
    [227, 149, 9],
    [229, 146, 9],
    [231, 142, 9],
    [235, 139, 9],
    [237, 136, 9],
    [239, 133, 9],
    [241, 130, 9],
    [241, 127, 9],
    [244, 122, 8],
    [246, 119, 8],
    [246, 117, 8],
    [248, 112, 8],
    [248, 109, 7],
    [250, 105, 7],
    [250, 103, 7],
    [250, 99, 6],
    [250, 95, 6],
    [252, 92, 6],
    [252, 88, 5],
    [252, 85, 5],
    [252, 81, 5],
    [250, 78, 5],
    [250, 74, 4],
    [250, 71, 4],
    [250, 68, 4],
    [248, 65, 3],
    [248, 61, 3],
    [246, 58, 3],
    [246, 55, 3],
    [244, 53, 2],
    [244, 50, 2],
    [241, 47, 2],
    [239, 45, 1],
    [237, 42, 1],
    [237, 40, 1],
    [235, 37, 1],
    [233, 35, 1],
    [231, 33, 1],
    [229, 31, 1],
    [227, 29, 0],
    [225, 27, 0],
    [221, 25, 0],
    [219, 24, 0],
    [217, 22, 0],
    [215, 21, 0],
    [211, 19, 0],
    [209, 18, 0],
    [207, 17, 0],
    [203, 16, 0],
    [201, 14, 0],
    [197, 13, 0],
    [195, 13, 0],
    [191, 12, 0],
    [187, 11, 0],
    [186, 10, 0],
    [182, 9, 0],
    [178, 9, 0],
    [175, 8, 0],
    [173, 7, 0],
    [169, 7, 0],
    [166, 6, 0],
    [162, 6, 0],
    [159, 5, 0],
    [154, 5, 0],
    [151, 4, 0],
    [147, 4, 0],
    [144, 3, 0],
    [141, 3, 0],
    [136, 3, 0],
    [133, 2, 0],
    [128, 2, 0],
    [125, 2, 0],
    [121, 1, 0],
    [118, 1, 0],
    [114, 1, 0],
    [109, 1, 0],
    [107, 1, 0],
    [103, 1, 0],
    [99, 0, 0],
    [95, 0, 0],
    [91, 0, 0],
    [87, 0, 0],
    [84, 0, 0],
    [80, 0, 0],
    [77, 0, 0],
    [73, 0, 0],
    [70, 0, 0],
    [67, 0, 0],
    [62, 0, 0],
    [59, 0, 0],
    [56, 0, 0],
    [53, 0, 0],
    [50, 0, 0],
];

// https://github.com/BIDS/colormap/blob/master/colormaps.py
const INFERNO: [[u8; 3]; 256] = [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 1],
    [0, 0, 1],
    [0, 0, 1],
    [0, 0, 2],
    [0, 0, 2],
    [0, 0, 2],
    [0, 0, 3],
    [0, 0, 3],
    [0, 0, 4],
    [0, 0, 4],
    [0, 0, 5],
    [0, 0, 6],
    [0, 0, 7],
    [0, 0, 7],
    [0, 0, 8],
    [1, 0, 9],
    [1, 0, 10],
    [1, 0, 11],
    [1, 0, 12],
    [1, 0, 13],
    [2, 0, 14],
    [2, 0, 15],
    [2, 0, 16],
    [3, 0, 17],
    [3, 0, 18],
    [3, 0, 19],
    [4, 0, 21],
    [4, 0, 22],
    [5, 0, 23],
    [5, 0, 24],
    [5, 0, 25],
    [6, 0, 27],
    [7, 0, 27],
    [7, 0, 29],
    [8, 0, 29],
    [8, 0, 30],
    [9, 0, 31],
    [10, 0, 32],
    [10, 0, 33],
    [11, 0, 33],
    [12, 0, 33],
    [12, 0, 34],
    [13, 0, 35],
    [14, 0, 36],
    [14, 0, 36],
    [15, 0, 36],
    [16, 0, 36],
    [17, 0, 37],
    [18, 0, 37],
    [19, 0, 38],
    [19, 0, 38],
    [21, 0, 38],
    [21, 0, 39],
    [22, 0, 39],
    [23, 0, 39],
    [24, 0, 39],
    [25, 0, 39],
    [26, 0, 40],
    [27, 0, 40],
    [29, 0, 40],
    [29, 0, 40],
    [31, 0, 40],
    [31, 0, 40],
    [33, 1, 40],
    [33, 1, 40],
    [35, 1, 40],
    [36, 1, 40],
    [37, 1, 40],
    [39, 1, 40],
    [40, 1, 40],
    [41, 1, 40],
    [43, 1, 39],
    [44, 1, 39],
    [45, 1, 39],
    [46, 1, 39],
    [48, 1, 39],
    [50, 1, 39],
    [51, 2, 38],
    [53, 2, 38],
    [54, 2, 38],
    [55, 2, 37],
    [56, 2, 37],
    [58, 2, 37],
    [60, 2, 36],
    [61, 2, 36],
    [63, 2, 36],
    [65, 3, 36],
    [67, 3, 36],
    [69, 3, 36],
    [70, 3, 35],
    [72, 3, 35],
    [73, 3, 34],
    [75, 3, 34],
    [78, 3, 33],
    [79, 3, 33],
    [81, 4, 33],
    [82, 4, 32],
    [85, 4, 32],
    [86, 4, 31],
    [88, 4, 31],
    [91, 4, 31],
    [92, 5, 30],
    [95, 5, 30],
    [96, 5, 29],
    [99, 5, 29],
    [100, 5, 29],
    [103, 5, 28],
    [105, 5, 27],
    [107, 6, 27],
    [109, 6, 26],
    [111, 6, 26],
    [114, 6, 25],
    [115, 7, 25],
    [118, 7, 24],
    [119, 7, 23],
    [122, 7, 23],
    [124, 8, 23],
    [127, 8, 22],
    [128, 8, 22],
    [131, 8, 21],
    [133, 9, 21],
    [135, 9, 20],
    [138, 9, 19],
    [139, 10, 19],
    [142, 10, 18],
    [144, 10, 18],
    [147, 11, 17],
    [149, 11, 17],
    [151, 11, 16],
    [154, 12, 16],
    [156, 12, 15],
    [157, 13, 15],
    [161, 13, 14],
    [162, 14, 13],
    [164, 14, 13],
    [166, 15, 13],
    [169, 15, 12],
    [171, 16, 12],
    [173, 16, 11],
    [175, 17, 11],
    [178, 18, 10],
    [180, 18, 10],
    [182, 19, 9],
    [184, 19, 9],
    [186, 21, 9],
    [187, 21, 8],
    [189, 22, 8],
    [191, 23, 7],
    [195, 23, 7],
    [197, 24, 7],
    [199, 25, 6],
    [201, 26, 6],
    [203, 27, 5],
    [203, 28, 5],
    [205, 29, 5],
    [207, 30, 5],
    [209, 31, 4],
    [211, 32, 4],
    [213, 33, 4],
    [215, 34, 3],
    [217, 35, 3],
    [217, 36, 3],
    [219, 38, 3],
    [221, 39, 2],
    [223, 40, 2],
    [223, 41, 2],
    [225, 43, 2],
    [227, 45, 1],
    [227, 45, 1],
    [229, 47, 1],
    [229, 49, 1],
    [231, 50, 1],
    [233, 52, 1],
    [233, 54, 0],
    [235, 55, 0],
    [235, 56, 0],
    [237, 58, 0],
    [237, 60, 0],
    [239, 62, 0],
    [239, 63, 0],
    [239, 66, 0],
    [241, 68, 0],
    [241, 70, 0],
    [241, 72, 0],
    [244, 73, 0],
    [244, 75, 0],
    [244, 78, 0],
    [244, 80, 0],
    [246, 82, 0],
    [246, 85, 0],
    [246, 87, 0],
    [246, 88, 0],
    [246, 91, 0],
    [246, 93, 0],
    [246, 96, 0],
    [246, 99, 0],
    [246, 101, 0],
    [246, 104, 0],
    [246, 107, 0],
    [246, 109, 0],
    [246, 112, 0],
    [246, 114, 1],
    [246, 117, 1],
    [246, 119, 1],
    [246, 122, 1],
    [246, 125, 2],
    [244, 128, 2],
    [244, 131, 3],
    [244, 135, 3],
    [244, 138, 4],
    [241, 141, 4],
    [241, 144, 5],
    [241, 147, 6],
    [239, 151, 6],
    [239, 154, 7],
    [239, 157, 8],
    [237, 161, 9],
    [237, 164, 10],
    [235, 168, 11],
    [235, 171, 13],
    [233, 175, 14],
    [233, 178, 15],
    [231, 182, 17],
    [231, 184, 19],
    [229, 187, 21],
    [229, 191, 23],
    [229, 195, 25],
    [227, 199, 27],
    [227, 203, 29],
    [225, 207, 32],
    [225, 209, 35],
    [225, 213, 38],
    [225, 217, 41],
    [225, 219, 45],
    [225, 223, 49],
    [225, 227, 53],
    [227, 229, 56],
    [227, 231, 60],
    [229, 235, 65],
    [231, 237, 69],
    [233, 239, 73],
    [235, 244, 78],
    [237, 246, 82],
    [241, 248, 87],
    [244, 250, 91],
    [248, 252, 96],
];

const PLASMA: [[u8; 3]; 256] = [
    [0, 0, 61],
    [0, 0, 62],
    [0, 0, 65],
    [1, 0, 66],
    [1, 0, 67],
    [1, 0, 68],
    [2, 0, 69],
    [2, 0, 70],
    [2, 0, 71],
    [3, 0, 72],
    [3, 0, 73],
    [4, 0, 74],
    [4, 0, 75],
    [5, 0, 77],
    [5, 0, 77],
    [6, 0, 78],
    [6, 0, 79],
    [7, 0, 80],
    [7, 0, 81],
    [8, 0, 81],
    [9, 0, 82],
    [9, 0, 84],
    [10, 0, 84],
    [10, 0, 85],
    [11, 0, 86],
    [12, 0, 86],
    [13, 0, 87],
    [13, 0, 88],
    [14, 0, 88],
    [15, 0, 90],
    [16, 0, 90],
    [16, 0, 91],
    [17, 0, 92],
    [18, 0, 92],
    [19, 0, 93],
    [20, 0, 93],
    [21, 0, 95],
    [22, 0, 95],
    [23, 0, 95],
    [23, 0, 96],
    [25, 0, 96],
    [25, 0, 97],
    [27, 0, 97],
    [28, 0, 97],
    [29, 0, 99],
    [30, 0, 99],
    [31, 0, 99],
    [32, 0, 100],
    [33, 0, 100],
    [34, 0, 100],
    [35, 0, 100],
    [36, 0, 100],
    [38, 0, 101],
    [39, 0, 101],
    [40, 0, 101],
    [41, 0, 101],
    [43, 0, 101],
    [44, 0, 101],
    [45, 0, 101],
    [46, 0, 101],
    [48, 0, 101],
    [49, 0, 101],
    [51, 0, 101],
    [52, 0, 100],
    [54, 0, 100],
    [55, 0, 100],
    [56, 0, 100],
    [57, 0, 100],
    [59, 0, 99],
    [60, 0, 99],
    [61, 0, 99],
    [63, 0, 97],
    [65, 0, 97],
    [67, 0, 96],
    [68, 0, 96],
    [70, 0, 96],
    [71, 0, 95],
    [72, 0, 95],
    [74, 0, 93],
    [75, 0, 92],
    [78, 0, 92],
    [79, 0, 91],
    [80, 0, 91],
    [82, 0, 90],
    [84, 1, 88],
    [85, 1, 88],
    [87, 1, 87],
    [88, 1, 86],
    [90, 1, 85],
    [91, 1, 85],
    [93, 1, 84],
    [95, 2, 82],
    [96, 2, 81],
    [97, 2, 80],
    [100, 2, 80],
    [101, 3, 79],
    [103, 3, 78],
    [104, 3, 77],
    [107, 3, 75],
    [108, 3, 74],
    [109, 4, 73],
    [111, 4, 72],
    [112, 4, 71],
    [114, 5, 71],
    [115, 5, 70],
    [118, 5, 69],
    [119, 5, 68],
    [121, 6, 67],
    [122, 6, 66],
    [124, 7, 65],
    [125, 7, 63],
    [127, 7, 62],
    [128, 8, 61],
    [130, 8, 60],
    [131, 8, 59],
    [133, 9, 58],
    [135, 9, 57],
    [136, 10, 56],
    [138, 10, 55],
    [139, 10, 55],
    [141, 11, 55],
    [142, 11, 54],
    [144, 12, 53],
    [146, 12, 52],
    [147, 13, 51],
    [149, 13, 50],
    [151, 14, 49],
    [152, 14, 48],
    [154, 15, 47],
    [156, 15, 46],
    [157, 16, 45],
    [159, 16, 45],
    [161, 17, 45],
    [162, 18, 44],
    [164, 18, 43],
    [164, 19, 42],
    [166, 19, 41],
    [168, 20, 40],
    [169, 21, 40],
    [171, 21, 39],
    [173, 22, 39],
    [175, 23, 38],
    [175, 23, 37],
    [176, 24, 36],
    [178, 25, 36],
    [180, 25, 35],
    [182, 26, 34],
    [184, 27, 33],
    [184, 28, 33],
    [186, 29, 33],
    [187, 29, 32],
    [189, 30, 31],
    [189, 31, 31],
    [191, 32, 30],
    [193, 33, 29],
    [195, 33, 29],
    [197, 34, 29],
    [197, 35, 28],
    [199, 36, 27],
    [201, 37, 27],
    [201, 38, 26],
    [203, 39, 25],
    [205, 40, 25],
    [207, 41, 25],
    [207, 42, 24],
    [209, 43, 23],
    [211, 44, 23],
    [211, 45, 22],
    [213, 46, 22],
    [215, 47, 22],
    [215, 48, 21],
    [217, 49, 21],
    [217, 51, 20],
    [219, 52, 19],
    [221, 53, 19],
    [221, 54, 18],
    [223, 55, 18],
    [223, 56, 18],
    [225, 57, 17],
    [227, 59, 17],
    [227, 60, 16],
    [229, 61, 16],
    [229, 62, 15],
    [231, 65, 15],
    [231, 66, 15],
    [233, 67, 14],
    [233, 69, 14],
    [235, 70, 13],
    [235, 71, 13],
    [235, 73, 13],
    [237, 74, 12],
    [237, 75, 12],
    [239, 78, 12],
    [239, 79, 11],
    [239, 81, 11],
    [241, 82, 10],
    [241, 84, 10],
    [244, 86, 10],
    [244, 87, 9],
    [244, 90, 9],
    [244, 91, 9],
    [246, 93, 9],
    [246, 95, 8],
    [246, 96, 8],
    [248, 99, 8],
    [248, 100, 8],
    [248, 103, 7],
    [248, 104, 7],
    [248, 107, 7],
    [248, 108, 6],
    [250, 111, 6],
    [250, 112, 6],
    [250, 115, 6],
    [250, 117, 5],
    [250, 119, 5],
    [250, 121, 5],
    [250, 124, 5],
    [250, 125, 5],
    [250, 128, 5],
    [250, 130, 4],
    [250, 133, 4],
    [250, 136, 4],
    [250, 138, 4],
    [250, 141, 4],
    [250, 142, 4],
    [250, 146, 3],
    [248, 147, 3],
    [248, 151, 3],
    [248, 154, 3],
    [248, 156, 3],
    [248, 159, 3],
    [246, 162, 3],
    [246, 164, 3],
    [246, 168, 3],
    [244, 171, 3],
    [244, 173, 3],
    [244, 176, 3],
    [241, 178, 3],
    [241, 182, 3],
    [239, 186, 3],
    [239, 189, 3],
    [237, 191, 3],
    [237, 195, 3],
    [235, 199, 3],
    [235, 201, 3],
    [233, 205, 3],
    [233, 209, 3],
    [231, 211, 3],
    [229, 215, 3],
    [229, 219, 3],
    [227, 223, 3],
    [227, 225, 3],
    [225, 229, 3],
    [223, 233, 3],
    [223, 235, 3],
    [221, 239, 2],
];

const MAGMA: [[u8; 3]; 256] = [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 1],
    [0, 0, 1],
    [0, 0, 1],
    [0, 0, 1],
    [0, 0, 2],
    [0, 0, 2],
    [0, 0, 3],
    [0, 0, 3],
    [0, 0, 3],
    [0, 0, 4],
    [0, 0, 4],
    [0, 0, 5],
    [0, 0, 6],
    [0, 0, 6],
    [0, 0, 7],
    [0, 0, 8],
    [1, 0, 9],
    [1, 0, 9],
    [1, 0, 10],
    [1, 0, 11],
    [1, 0, 12],
    [1, 0, 13],
    [1, 0, 14],
    [2, 0, 16],
    [2, 0, 17],
    [2, 0, 18],
    [3, 0, 19],
    [3, 0, 21],
    [3, 0, 22],
    [3, 0, 23],
    [4, 0, 25],
    [4, 0, 27],
    [5, 0, 28],
    [5, 0, 29],
    [6, 0, 31],
    [6, 0, 33],
    [7, 0, 34],
    [7, 0, 35],
    [8, 0, 36],
    [8, 0, 38],
    [9, 0, 40],
    [10, 0, 40],
    [10, 0, 42],
    [11, 0, 43],
    [12, 0, 44],
    [13, 0, 45],
    [13, 0, 45],
    [14, 0, 46],
    [15, 0, 47],
    [15, 0, 48],
    [16, 0, 49],
    [17, 0, 49],
    [18, 0, 50],
    [19, 0, 51],
    [19, 0, 51],
    [21, 0, 52],
    [21, 0, 52],
    [22, 0, 53],
    [23, 0, 53],
    [24, 1, 54],
    [25, 1, 54],
    [26, 1, 54],
    [27, 1, 54],
    [28, 1, 55],
    [29, 1, 55],
    [30, 1, 55],
    [31, 1, 55],
    [33, 1, 55],
    [33, 1, 55],
    [35, 1, 55],
    [36, 1, 55],
    [37, 1, 55],
    [38, 2, 55],
    [40, 2, 56],
    [40, 2, 56],
    [42, 2, 56],
    [44, 2, 56],
    [45, 2, 56],
    [46, 2, 56],
    [47, 2, 56],
    [49, 3, 56],
    [50, 3, 56],
    [52, 3, 56],
    [54, 3, 56],
    [55, 3, 56],
    [56, 3, 56],
    [57, 3, 56],
    [59, 3, 56],
    [60, 3, 56],
    [62, 4, 56],
    [65, 4, 56],
    [66, 4, 56],
    [68, 4, 55],
    [69, 4, 55],
    [71, 4, 55],
    [73, 4, 55],
    [74, 5, 55],
    [77, 5, 55],
    [78, 5, 55],
    [80, 5, 55],
    [82, 5, 55],
    [84, 5, 55],
    [86, 5, 55],
    [88, 5, 54],
    [90, 6, 54],
    [92, 6, 54],
    [95, 6, 54],
    [96, 6, 53],
    [99, 6, 53],
    [100, 6, 53],
    [103, 7, 52],
    [105, 7, 52],
    [107, 7, 51],
    [109, 7, 51],
    [112, 7, 51],
    [114, 8, 50],
    [117, 8, 50],
    [119, 8, 49],
    [121, 8, 49],
    [124, 8, 48],
    [125, 8, 48],
    [128, 9, 47],
    [131, 9, 47],
    [133, 9, 46],
    [136, 9, 45],
    [139, 9, 45],
    [141, 10, 45],
    [144, 10, 45],
    [146, 10, 44],
    [149, 10, 43],
    [152, 11, 43],
    [154, 11, 42],
    [157, 11, 41],
    [159, 12, 41],
    [162, 12, 40],
    [164, 13, 40],
    [168, 13, 39],
    [169, 13, 39],
    [173, 13, 38],
    [175, 14, 37],
    [178, 14, 36],
    [180, 15, 36],
    [184, 15, 36],
    [186, 16, 35],
    [187, 16, 34],
    [191, 17, 33],
    [193, 17, 33],
    [195, 18, 33],
    [199, 18, 32],
    [201, 19, 31],
    [203, 20, 31],
    [205, 21, 31],
    [207, 22, 30],
    [211, 22, 29],
    [213, 23, 29],
    [215, 24, 29],
    [217, 25, 29],
    [219, 26, 28],
    [219, 27, 27],
    [221, 28, 27],
    [223, 29, 27],
    [225, 30, 27],
    [227, 31, 27],
    [229, 33, 27],
    [229, 34, 26],
    [231, 35, 26],
    [233, 36, 26],
    [233, 38, 26],
    [235, 40, 26],
    [235, 41, 26],
    [237, 42, 26],
    [237, 44, 27],
    [239, 45, 27],
    [239, 47, 27],
    [241, 49, 27],
    [241, 51, 27],
    [241, 53, 27],
    [244, 55, 28],
    [244, 55, 28],
    [244, 57, 29],
    [246, 59, 29],
    [246, 61, 29],
    [246, 63, 30],
    [246, 66, 31],
    [248, 68, 31],
    [248, 70, 31],
    [248, 72, 32],
    [248, 74, 33],
    [248, 75, 33],
    [250, 78, 34],
    [250, 80, 35],
    [250, 82, 36],
    [250, 85, 36],
    [250, 87, 37],
    [250, 90, 38],
    [250, 92, 40],
    [250, 93, 40],
    [250, 96, 41],
    [252, 99, 42],
    [252, 101, 44],
    [252, 104, 45],
    [252, 107, 45],
    [252, 109, 46],
    [252, 111, 48],
    [252, 114, 49],
    [252, 117, 51],
    [252, 119, 52],
    [252, 122, 53],
    [252, 125, 55],
    [252, 128, 55],
    [252, 130, 57],
    [252, 133, 58],
    [252, 136, 60],
    [252, 139, 61],
    [252, 142, 63],
    [252, 146, 65],
    [252, 147, 67],
    [252, 151, 69],
    [252, 154, 70],
    [250, 157, 72],
    [250, 161, 74],
    [250, 164, 75],
    [250, 166, 78],
    [250, 169, 80],
    [250, 173, 81],
    [250, 176, 84],
    [250, 180, 86],
    [250, 184, 87],
    [250, 186, 90],
    [250, 189, 92],
    [250, 193, 95],
    [248, 197, 97],
    [248, 201, 99],
    [248, 203, 101],
    [248, 207, 104],
    [248, 211, 107],
    [248, 215, 109],
    [248, 219, 112],
    [248, 223, 114],
    [248, 225, 117],
    [248, 229, 119],
    [248, 233, 122],
    [246, 237, 125],
    [246, 241, 128],
    [246, 244, 131],
    [246, 248, 135],
];

const VIRIDIS: [[u8; 3]; 256] = [
    [13, 0, 22],
    [13, 0, 22],
    [13, 0, 23],
    [14, 0, 24],
    [14, 0, 25],
    [14, 0, 26],
    [14, 0, 27],
    [14, 0, 28],
    [14, 0, 29],
    [14, 0, 30],
    [15, 0, 31],
    [15, 0, 31],
    [15, 0, 33],
    [15, 0, 33],
    [15, 1, 34],
    [15, 1, 36],
    [15, 1, 36],
    [15, 1, 37],
    [15, 1, 38],
    [15, 1, 40],
    [15, 2, 40],
    [15, 2, 41],
    [15, 2, 42],
    [15, 2, 43],
    [15, 3, 44],
    [15, 3, 45],
    [15, 3, 45],
    [15, 3, 46],
    [15, 4, 47],
    [15, 4, 48],
    [15, 4, 49],
    [15, 5, 50],
    [15, 5, 51],
    [14, 5, 52],
    [14, 6, 52],
    [14, 6, 53],
    [14, 6, 54],
    [14, 7, 55],
    [14, 7, 55],
    [14, 8, 55],
    [14, 8, 56],
    [13, 8, 56],
    [13, 9, 57],
    [13, 9, 58],
    [13, 10, 58],
    [13, 10, 59],
    [13, 10, 59],
    [13, 11, 60],
    [13, 12, 60],
    [12, 12, 61],
    [12, 13, 61],
    [12, 13, 62],
    [12, 13, 62],
    [11, 14, 62],
    [11, 15, 63],
    [11, 15, 63],
    [11, 16, 65],
    [10, 16, 65],
    [10, 17, 65],
    [10, 17, 65],
    [10, 18, 66],
    [10, 18, 66],
    [10, 19, 66],
    [10, 20, 66],
    [9, 21, 67],
    [9, 21, 67],
    [9, 22, 67],
    [9, 22, 67],
    [9, 23, 67],
    [9, 23, 68],
    [8, 24, 68],
    [8, 25, 68],
    [8, 25, 68],
    [8, 26, 68],
    [8, 27, 68],
    [8, 27, 68],
    [7, 28, 69],
    [7, 29, 69],
    [7, 29, 69],
    [7, 30, 69],
    [7, 31, 69],
    [7, 31, 69],
    [6, 32, 69],
    [6, 33, 69],
    [6, 33, 69],
    [6, 34, 69],
    [6, 35, 69],
    [6, 36, 69],
    [6, 36, 69],
    [5, 37, 70],
    [5, 38, 70],
    [5, 39, 70],
    [5, 40, 70],
    [5, 40, 70],
    [5, 41, 70],
    [5, 42, 70],
    [5, 43, 70],
    [5, 44, 70],
    [5, 45, 70],
    [4, 45, 70],
    [4, 46, 70],
    [4, 47, 70],
    [4, 48, 70],
    [4, 49, 70],
    [4, 50, 70],
    [4, 50, 70],
    [4, 51, 70],
    [4, 52, 70],
    [4, 53, 70],
    [4, 54, 70],
    [3, 55, 70],
    [3, 55, 70],
    [3, 56, 70],
    [3, 57, 70],
    [3, 58, 69],
    [3, 59, 69],
    [3, 60, 69],
    [3, 61, 69],
    [3, 62, 69],
    [3, 63, 69],
    [3, 65, 69],
    [3, 65, 69],
    [3, 66, 69],
    [3, 67, 69],
    [2, 68, 69],
    [2, 69, 68],
    [2, 70, 68],
    [2, 71, 68],
    [2, 72, 68],
    [2, 73, 68],
    [2, 74, 68],
    [2, 75, 67],
    [2, 77, 67],
    [2, 78, 67],
    [2, 79, 67],
    [2, 80, 66],
    [2, 81, 66],
    [2, 82, 66],
    [2, 82, 66],
    [2, 84, 65],
    [2, 85, 65],
    [2, 86, 65],
    [2, 87, 63],
    [2, 88, 63],
    [2, 90, 63],
    [2, 91, 62],
    [2, 92, 62],
    [2, 93, 61],
    [2, 95, 61],
    [2, 96, 60],
    [2, 97, 60],
    [2, 99, 60],
    [2, 100, 59],
    [3, 100, 59],
    [3, 101, 58],
    [3, 103, 57],
    [3, 104, 57],
    [3, 105, 56],
    [3, 107, 56],
    [4, 108, 55],
    [4, 109, 55],
    [4, 111, 55],
    [4, 112, 54],
    [5, 114, 53],
    [5, 114, 53],
    [5, 115, 52],
    [6, 117, 51],
    [6, 118, 50],
    [7, 119, 50],
    [7, 121, 49],
    [8, 122, 48],
    [8, 124, 47],
    [9, 125, 46],
    [9, 125, 46],
    [10, 127, 45],
    [10, 128, 45],
    [11, 130, 44],
    [12, 131, 43],
    [13, 133, 42],
    [13, 133, 41],
    [14, 135, 40],
    [15, 136, 40],
    [16, 138, 39],
    [17, 139, 38],
    [18, 139, 37],
    [19, 141, 36],
    [20, 142, 35],
    [21, 144, 34],
    [22, 146, 33],
    [23, 146, 33],
    [25, 147, 32],
    [26, 149, 31],
    [28, 151, 30],
    [29, 151, 29],
    [31, 152, 29],
    [32, 154, 27],
    [34, 156, 27],
    [36, 156, 26],
    [37, 157, 25],
    [39, 159, 24],
    [41, 159, 23],
    [43, 161, 22],
    [45, 162, 22],
    [47, 162, 21],
    [49, 164, 20],
    [52, 166, 19],
    [54, 166, 18],
    [56, 168, 17],
    [58, 168, 17],
    [61, 169, 16],
    [63, 171, 15],
    [67, 171, 14],
    [69, 173, 13],
    [72, 173, 13],
    [74, 175, 12],
    [78, 175, 11],
    [80, 176, 11],
    [84, 176, 10],
    [87, 178, 9],
    [90, 178, 9],
    [93, 180, 8],
    [97, 180, 8],
    [100, 182, 7],
    [104, 182, 7],
    [108, 184, 6],
    [111, 184, 5],
    [115, 186, 5],
    [119, 186, 5],
    [122, 186, 4],
    [127, 187, 4],
    [131, 187, 3],
    [135, 189, 3],
    [139, 189, 3],
    [144, 189, 2],
    [147, 191, 2],
    [152, 191, 2],
    [157, 191, 2],
    [161, 193, 1],
    [166, 193, 1],
    [169, 193, 1],
    [175, 195, 1],
    [180, 195, 1],
    [184, 195, 1],
    [189, 197, 1],
    [193, 197, 1],
    [199, 197, 1],
    [205, 199, 1],
    [209, 199, 1],
    [215, 199, 1],
    [219, 201, 1],
    [225, 201, 1],
    [229, 201, 2],
    [235, 203, 2],
    [239, 203, 2],
    [244, 203, 3],
    [250, 205, 3],
];

const BW: [[u8; 3]; 256] = [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [6, 6, 6],
    [12, 12, 12],
    [19, 19, 19],
    [25, 25, 25],
    [31, 31, 31],
    [38, 38, 38],
    [44, 44, 44],
    [51, 51, 51],
    [57, 57, 57],
    [63, 63, 63],
    [70, 70, 70],
    [76, 76, 76],
    [82, 82, 82],
    [89, 89, 89],
    [95, 95, 95],
    [102, 102, 102],
    [108, 108, 108],
    [114, 114, 114],
    [121, 121, 121],
    [127, 127, 127],
    [133, 133, 133],
    [140, 140, 140],
    [146, 146, 146],
    [153, 153, 153],
    [159, 159, 159],
    [165, 165, 165],
    [172, 172, 172],
    [178, 178, 178],
    [184, 184, 184],
    [191, 191, 191],
    [197, 197, 197],
    [204, 204, 204],
    [210, 210, 210],
    [216, 216, 216],
    [223, 223, 223],
    [229, 229, 229],
    [235, 235, 235],
    [242, 242, 242],
    [248, 248, 248],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
    [255, 255, 255],
];
