pub mod matrix_3x2_f {
  use winapi::um::{d2d1::D2D1_MATRIX_3X2_F, dwrite};
  pub fn id() -> D2D1_MATRIX_3X2_F {
    D2D1_MATRIX_3X2_F {
      matrix: [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
    }
  }

  pub fn from_dwrite_matrix(dw: &dwrite::DWRITE_MATRIX) -> D2D1_MATRIX_3X2_F {
    D2D1_MATRIX_3X2_F {
      matrix: [[dw.m11, dw.m12], [dw.m21, dw.m22], [dw.dx, dw.dy]],
    }
  }

  pub fn translate(dx: f32, dy: f32) -> D2D1_MATRIX_3X2_F {
    D2D1_MATRIX_3X2_F {
      matrix: [[1.0, 0.0], [0.0, 1.0], [dx, dy]],
    }
  }

  pub fn mul(l: &D2D1_MATRIX_3X2_F, r: &D2D1_MATRIX_3X2_F) -> D2D1_MATRIX_3X2_F {
    let a = &l.matrix;
    let b = &r.matrix;

    D2D1_MATRIX_3X2_F {
      matrix: [
        [
          a[0][0] * b[0][0] + a[0][1] * b[1][0],
          a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
          a[1][0] * b[0][0] + a[1][1] * b[1][0],
          a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
        [
          a[2][0] * b[0][0] + a[2][1] * b[1][0] + b[2][0],
          a[2][0] * b[0][1] + a[2][1] * b[1][1] + b[2][1],
        ],
      ],
    }
  }
}
