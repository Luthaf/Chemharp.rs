// Chemfiles, a modern library for chemistry file reading and writing
// Copyright (C) 2015-2018 Guillaume Fraux -- BSD licensed
use std::ops::{Drop, Deref, DerefMut};
use std::marker::PhantomData;

use chemfiles_sys::*;
use errors::{check, Error};
use Result;

/// Available unit cell shapes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(stutter)]
pub enum CellShape {
    /// Orthorhombic cell, with the three angles equals to 90°.
    Orthorhombic,
    /// Triclinic cell, with any values for the angles.
    Triclinic,
    /// Infinite cell, to use when there is no cell.
    Infinite,
}

impl From<chfl_cellshape> for CellShape {
    fn from(celltype: chfl_cellshape) -> CellShape {
        match celltype {
            chfl_cellshape::CHFL_CELL_ORTHORHOMBIC => CellShape::Orthorhombic,
            chfl_cellshape::CHFL_CELL_TRICLINIC => CellShape::Triclinic,
            chfl_cellshape::CHFL_CELL_INFINITE => CellShape::Infinite,
        }
    }
}

impl From<CellShape> for chfl_cellshape {
    fn from(celltype: CellShape) -> chfl_cellshape {
        match celltype {
            CellShape::Orthorhombic => chfl_cellshape::CHFL_CELL_ORTHORHOMBIC,
            CellShape::Triclinic => chfl_cellshape::CHFL_CELL_TRICLINIC,
            CellShape::Infinite => chfl_cellshape::CHFL_CELL_INFINITE,
        }
    }
}

/// An `UnitCell` represent the box containing the UnitCells, and its periodicity.
///
/// An unit cell is fully represented by three lengths (a, b, c); and three
/// angles (alpha, beta, gamma). The angles are stored in degrees, and the
/// lengths in Angstroms.
///
/// A cell also has a matricial representation, by projecting the three base
/// vector into an orthonormal base. We choose to represent such matrix as an
/// upper triangular matrix:
///
/// ```text
/// | a_x   b_x   c_x |
/// |  0    b_y   c_y |
/// |  0     0    c_z |
/// ```
#[allow(stutter)]
pub struct UnitCell {
    handle: *mut CHFL_CELL,
}

/// An analog to a reference to an unit cell (`&UnitCell`)
pub struct UnitCellRef<'a> {
    inner: UnitCell,
    marker: PhantomData<&'a UnitCell>
}

impl<'a> Deref for UnitCellRef<'a> {
    type Target = UnitCell;
    fn deref(&self) -> &UnitCell {
        &self.inner
    }
}

/// An analog to a mutable reference to an unit cell (`&mut UnitCell`)
pub struct UnitCellMut<'a> {
    inner: UnitCell,
    marker: PhantomData<&'a mut UnitCell>
}

impl<'a> Deref for UnitCellMut<'a> {
    type Target = UnitCell;
    fn deref(&self) -> &UnitCell {
        &self.inner
    }
}

impl<'a> DerefMut for UnitCellMut<'a> {
    fn deref_mut(&mut self) -> &mut UnitCell {
        &mut self.inner
    }
}

impl Clone for UnitCell {
    fn clone(&self) -> UnitCell {
        unsafe {
            let new_handle = chfl_cell_copy(self.as_ptr());
            UnitCell::from_ptr(new_handle).expect("Out of memory when copying an UnitCell")
        }
    }
}

impl UnitCell {
    /// Create an owned `UnitCell` from a C pointer.
    ///
    /// This function is unsafe because no validity check is made on the pointer,
    /// except for it being non-null.
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *mut CHFL_CELL) -> Result<UnitCell> {
        if ptr.is_null() {
            Err(Error::null_ptr())
        } else {
            Ok(UnitCell { handle: ptr })
        }
    }

    /// Create a borrowed `UnitCell` from a C pointer.
    ///
    /// This function is unsafe because no validity check is made on the
    /// pointer, except for it being non-null, and the caller is responsible for
    /// setting the right lifetime
    #[inline]
    pub(crate) unsafe fn ref_from_ptr<'a>(ptr: *const CHFL_CELL) -> Result<UnitCellRef<'a>> {
        let cell = try!(UnitCell::from_ptr(ptr as *mut CHFL_CELL));
        Ok(UnitCellRef {
            inner: cell,
            marker: PhantomData,
        })
    }

    /// Create a borrowed `UnitCell` from a C pointer.
    ///
    /// This function is unsafe because no validity check is made on the
    /// pointer, except for it being non-null, and the caller is responsible for
    /// setting the right lifetime
    #[inline]
    pub(crate) unsafe fn ref_mut_from_ptr<'a>(ptr: *mut CHFL_CELL) -> Result<UnitCellMut<'a>> {
        let cell = try!(UnitCell::from_ptr(ptr));
        Ok(UnitCellMut {
            inner: cell,
            marker: PhantomData,
        })
    }

    /// Get the underlying C pointer as a const pointer.
    #[inline]
    pub(crate) fn as_ptr(&self) -> *const CHFL_CELL {
        self.handle
    }

    /// Get the underlying C pointer as a mutable pointer.
    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut CHFL_CELL {
        self.handle
    }

    /// Create an `Orthorhombic` `UnitCell` from the three lengths, in Angstroms.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::{UnitCell, CellShape};
    /// let cell = UnitCell::new([30.0, 30.0, 23.0]).unwrap();
    ///
    /// assert_eq!(cell.lengths(), Ok([30.0, 30.0, 23.0]));
    /// assert_eq!(cell.angles(), Ok([90.0, 90.0, 90.0]));
    /// assert_eq!(cell.shape(), Ok(CellShape::Orthorhombic));
    /// ```
    pub fn new(lengths: [f64; 3]) -> Result<UnitCell> {
        unsafe {
            let handle = chfl_cell(lengths.as_ptr());
            UnitCell::from_ptr(handle)
        }
    }

    /// Create an `Infinite` `UnitCell`.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::{UnitCell, CellShape};
    /// let cell = UnitCell::infinite().unwrap();
    ///
    /// assert_eq!(cell.lengths(), Ok([0.0, 0.0, 0.0]));
    /// assert_eq!(cell.angles(), Ok([90.0, 90.0, 90.0]));
    /// assert_eq!(cell.shape(), Ok(CellShape::Infinite));
    /// ```
    pub fn infinite() -> Result<UnitCell> {
        let mut cell = UnitCell::new([0.0, 0.0, 0.0])?;
        cell.set_shape(CellShape::Infinite)?;
        Ok(cell)
    }

    /// Create an `Triclinic` `UnitCell` from the three lengths (in Angstroms)
    /// and three angles (in degree). `alpha` is the angle between the vectors
    /// `b` and `c`; `beta` is the between the vectors `a` and `c` and `gamma`
    /// is the angle between the vectors `a` and `b`.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::{UnitCell, CellShape};
    /// let cell = UnitCell::triclinic([10.0, 10.0, 10.0], [98.0, 99.0, 90.0]).unwrap();
    ///
    /// assert_eq!(cell.lengths(), Ok([10.0, 10.0, 10.0]));
    /// assert_eq!(cell.angles(), Ok([98.0, 99.0, 90.0]));
    /// assert_eq!(cell.shape(), Ok(CellShape::Triclinic));
    /// ```
    pub fn triclinic(lengths: [f64; 3], angles: [f64; 3]) -> Result<UnitCell> {
        unsafe {
            let handle = chfl_cell_triclinic(lengths.as_ptr(), angles.as_ptr());
            UnitCell::from_ptr(handle)
        }
    }

    /// Get the three lengths of the cell, in Angstroms.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let cell = UnitCell::new([30.0, 30.0, 23.0]).unwrap();
    /// assert_eq!(cell.lengths(), Ok([30.0, 30.0, 23.0]));
    /// ```
    pub fn lengths(&self) -> Result<[f64; 3]> {
        let mut lengths = [0.0; 3];
        unsafe {
            check(chfl_cell_lengths(self.as_ptr(), lengths.as_mut_ptr()))?;
        }
        Ok(lengths)
    }

    /// Set the three lengths of the cell, in Angstroms.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let mut cell = UnitCell::new([30.0, 30.0, 23.0]).unwrap();
    ///
    /// cell.set_lengths([10.0, 30.0, 42.0]).unwrap();
    /// assert_eq!(cell.lengths(), Ok([10.0, 30.0, 42.0]));
    /// ```
    pub fn set_lengths(&mut self, lengths: [f64; 3]) -> Result<()> {
        unsafe {
            check(chfl_cell_set_lengths(self.as_mut_ptr(), lengths.as_ptr()))?;
        }
        Ok(())
    }

    /// Get the three angles of the cell, in degrees.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let cell = UnitCell::new([20.0, 20.0, 20.0]).unwrap();
    /// assert_eq!(cell.angles(), Ok([90.0, 90.0, 90.0]));
    ///
    /// let cell = UnitCell::triclinic([20.0, 20.0, 20.0], [100.0, 120.0, 90.0]).unwrap();
    /// assert_eq!(cell.angles(), Ok([100.0, 120.0, 90.0]));
    /// ```
    pub fn angles(&self) -> Result<[f64; 3]> {
        let mut angles = [0.0; 3];
        unsafe {
            check(chfl_cell_angles(self.as_ptr(), angles.as_mut_ptr()))?;
        }
        Ok(angles)
    }

    /// Set the three angles of the cell, in degrees. This is only possible
    /// with `Triclinic` cells.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let mut cell = UnitCell::triclinic([20.0, 20.0, 20.0], [100.0, 120.0, 90.0]).unwrap();
    /// assert_eq!(cell.angles(), Ok([100.0, 120.0, 90.0]));
    ///
    /// cell.set_angles([90.0, 90.0, 90.0]).unwrap();
    /// assert_eq!(cell.angles(), Ok([90.0, 90.0, 90.0]));
    /// ```
    pub fn set_angles(&mut self, angles: [f64; 3]) -> Result<()> {
        unsafe {
            check(chfl_cell_set_angles(self.as_mut_ptr(), angles.as_ptr()))?;
        }
        Ok(())
    }

    /// Get the unit cell matricial representation.
    ///
    /// The unit cell representation is obtained by aligning the a vector along
    /// the *x* axis and putting the b vector in the *xy* plane. This make the
    /// matrix an upper triangular matrix:
    ///
    /// ```text
    /// | a_x   b_x   c_x |
    /// |  0    b_y   c_y |
    /// |  0     0    c_z |
    /// ```
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let mut cell = UnitCell::new([10.0, 20.0, 30.0]).unwrap();
    ///
    /// let matrix = cell.matrix().unwrap();
    ///
    /// assert_eq!(matrix[0][0], 10.0);
    /// assert_eq!(matrix[1][1], 20.0);
    /// assert_eq!(matrix[2][2], 30.0);
    ///
    /// assert!(matrix[1][2].abs() < 1e-9);
    /// ```
    pub fn matrix(&self) -> Result<[[f64; 3]; 3]> {
        let mut res = [[0.0; 3]; 3];
        unsafe {
            check(chfl_cell_matrix(self.as_ptr(), res.as_mut_ptr()))?;
        }
        Ok(res)
    }

    /// Get the shape of the unit cell.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::{UnitCell, CellShape};
    /// let cell = UnitCell::new([10.0, 20.0, 30.0]).unwrap();
    /// assert_eq!(cell.shape(), Ok(CellShape::Orthorhombic));
    /// ```
    pub fn shape(&self) -> Result<CellShape> {
        let mut shape = chfl_cellshape::CHFL_CELL_INFINITE;
        unsafe {
            check(chfl_cell_shape(self.as_ptr(), &mut shape))?;
        }
        Ok(CellShape::from(shape))
    }

    /// Set the shape of the unit cell to `shape`.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::{UnitCell, CellShape};
    /// let mut cell = UnitCell::new([10.0, 20.0, 30.0]).unwrap();
    /// assert_eq!(cell.shape(), Ok(CellShape::Orthorhombic));
    ///
    /// cell.set_shape(CellShape::Triclinic).unwrap();
    /// assert_eq!(cell.shape(), Ok(CellShape::Triclinic));
    /// ```
    pub fn set_shape(&mut self, shape: CellShape) -> Result<()> {
        unsafe {
            check(chfl_cell_set_shape(self.as_mut_ptr(), shape.into()))?;
        }
        Ok(())
    }

    /// Get the volume of the unit cell.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let cell = UnitCell::new([10.0, 20.0, 30.0]).unwrap();
    /// assert_eq!(cell.volume(), Ok(10.0 * 20.0 * 30.0));
    /// ```
    pub fn volume(&self) -> Result<f64> {
        let mut res = 0.0;
        unsafe {
            check(chfl_cell_volume(self.as_ptr(), &mut res))?;
        }
        Ok(res)
    }

    /// Wrap a `vector` in this unit cell.
    ///
    /// # Example
    /// ```
    /// # use chemfiles::UnitCell;
    /// let cell = UnitCell::new([10.0, 20.0, 30.0]).unwrap();
    ///
    /// let mut vector = [12.0, 5.2, -45.3];
    /// cell.wrap(&mut vector).unwrap();
    /// assert_eq!(vector, [2.0, 5.2, 14.700000000000003]);
    /// ```
    pub fn wrap(&self, vector: &mut [f64; 3]) -> Result<()> {
        unsafe { check(chfl_cell_wrap(self.as_ptr(), vector.as_mut_ptr())) }
    }
}

impl Drop for UnitCell {
    fn drop(&mut self) {
        unsafe {
            let status = chfl_cell_free(self.as_mut_ptr());
            debug_assert_eq!(status, chfl_status::CHFL_SUCCESS);
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clone() {
        let mut cell = UnitCell::new([2.0, 3.0, 4.0]).unwrap();
        assert_eq!(cell.lengths(), Ok([2.0, 3.0, 4.0]));

        let copy = cell.clone();
        assert_eq!(copy.lengths(), Ok([2.0, 3.0, 4.0]));

        assert!(cell.set_lengths([10.0, 12.0, 11.0]).is_ok());
        assert_eq!(cell.lengths(), Ok([10.0, 12.0, 11.0]));
        assert_eq!(copy.lengths(), Ok([2.0, 3.0, 4.0]));
    }

    #[test]
    fn lengths() {
        let mut cell = UnitCell::new([2.0, 3.0, 4.0]).unwrap();

        assert_eq!(cell.lengths(), Ok([2.0, 3.0, 4.0]));

        assert!(cell.set_lengths([10.0, 12.0, 11.0]).is_ok());
        assert_eq!(cell.lengths(), Ok([10.0, 12.0, 11.0]));
    }

    #[test]
    fn angles() {
        let mut cell = UnitCell::new([2.0, 3.0, 4.0]).unwrap();

        assert_eq!(cell.angles(), Ok([90.0, 90.0, 90.0]));

        assert!(cell.set_shape(CellShape::Triclinic).is_ok());
        assert!(cell.set_angles([80.0, 89.0, 100.0]).is_ok());

        assert_eq!(cell.angles(), Ok([80.0, 89.0, 100.0]));

        let cell = UnitCell::triclinic([1., 2., 3.], [80., 90., 100.]).unwrap();
        assert_eq!(cell.angles(), Ok([80.0, 90.0, 100.0]));
    }

    #[test]
    fn volume() {
        let cell = UnitCell::new([2.0, 3.0, 4.0]).unwrap();
        assert_eq!(cell.volume(), Ok(2.0 * 3.0 * 4.0));
    }

    #[test]
    fn wrap() {
        let cell = UnitCell::new([10.0, 20.0, 30.0]).unwrap();
        let mut vector = [12.0, 5.2, -45.3];
        cell.wrap(&mut vector).unwrap();
        assert_eq!(vector, [2.0, 5.2, 14.700000000000003]);
    }

    #[test]
    fn matrix() {
        let cell = UnitCell::new([2.0, 3.0, 4.0]).unwrap();

        let matrix = cell.matrix().unwrap();
        let result = [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]];

        for i in 0..3 {
            for j in 0..3 {
                assert_ulps_eq!(matrix[i][j], result[i][j], epsilon = 1e-12);
            }
        }
    }

    #[test]
    fn shape() {
        let cell = UnitCell::new([2.0, 3.0, 4.0]).unwrap();
        assert_eq!(cell.shape(), Ok(CellShape::Orthorhombic));

        let cell = UnitCell::infinite().unwrap();
        assert_eq!(cell.shape(), Ok(CellShape::Infinite));

        let cell = UnitCell::triclinic([1.0, 2.0, 3.0], [80.0, 90.0, 100.0]).unwrap();
        assert_eq!(cell.shape(), Ok(CellShape::Triclinic));

        let mut cell = UnitCell::new([0.0, 0.0, 0.0]).unwrap();
        assert_eq!(cell.shape(), Ok(CellShape::Orthorhombic));
        assert!(cell.set_shape(CellShape::Infinite).is_ok());
        assert_eq!(cell.shape(), Ok(CellShape::Infinite));
    }
}
