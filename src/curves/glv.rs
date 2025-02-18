use crate::{EdwardsParameters, Fq, Fr, FrParameters};
use ark_ec::{AffineCurve, ModelParameters, ProjectiveCurve};
use ark_ff::{field_new, BigInteger, BigInteger256, FpParameters, One};
use ark_std::{cmp::max, Zero};
use num_bigint::BigUint;

/// The GLV parameters that are useful to compute the endomorphism
/// and scalar decomposition.
pub trait GLVParameters: Send + Sync + 'static + ModelParameters {
    type CurveAffine;
    type CurveProjective;

    // phi(P) = lambda*P for all P
    // constants that are used to calculate phi(P)
    const COEFF_A1: Self::BaseField;
    const COEFF_A2: Self::BaseField;
    const COEFF_A3: Self::BaseField;
    const COEFF_B1: Self::BaseField;
    const COEFF_B2: Self::BaseField;
    const COEFF_B3: Self::BaseField;
    const COEFF_C1: Self::BaseField;
    const COEFF_C2: Self::BaseField;

    // constants that are used to perform scalar decomposition
    // This is a matrix which is practically the LLL reduced bases
    const COEFF_N11: Self::ScalarField;
    const COEFF_N12: Self::ScalarField;
    const COEFF_N21: Self::ScalarField;
    const COEFF_N22: Self::ScalarField;

    /// mapping a point G to phi(G):= lambda G where psi is the endomorphism
    fn endomorphism(base: &Self::CurveAffine) -> Self::CurveAffine;

    /// decompose a scalar s into k1, k2, s.t. s = k1 + lambda k2
    fn scalar_decomposition(
        k: &Self::ScalarField,
    ) -> (Self::ScalarField, Self::ScalarField);

    /// perform GLV multiplication
    fn glv_mul(
        base: &Self::CurveAffine,
        scalar: &Self::ScalarField,
    ) -> Self::CurveProjective;
}

impl GLVParameters for EdwardsParameters {
    type CurveAffine = crate::EdwardsAffine;
    type CurveProjective = crate::EdwardsProjective;

    // phi(P) = lambda*P for all P
    // constants that are used to calculate phi(P)
    const COEFF_A1: Self::BaseField = field_new!(
        Fq,
        "16179988757916560824577558193084210236647645729299773892093730683504906651604"
    );

    const COEFF_A2: Self::BaseField = field_new!(
        Fq,
        "37446463827641770816307242315180085052603635617490163568005256780843403514036"
    );

    const COEFF_A3: Self::BaseField = field_new!(
        Fq,
        "14989411347484419663140498193005880785086916883037474254598401919095177670477"
    );

    const COEFF_B1: Self::BaseField = field_new!(
        Fq,
        "37446463827641770816307242315180085052603635617490163568005256780843403514036"
    );

    const COEFF_B2: Self::BaseField = field_new!(
        Fq,
        "36553259151239542273674161596529768046449890757310263666255995151154432137034"
    );

    const COEFF_B3: Self::BaseField = field_new!(
        Fq,
        "15882616023886648205773578911656197791240661743217374156347663548784149047479"
    );

    const COEFF_C1: Self::BaseField = field_new!(
        Fq,
        "42910309089382041158038545419309140955400939872179826051492616687477682993077"
    );

    const COEFF_C2: Self::BaseField = field_new!(
        Fq,
        "9525566085744149321409195088876824882289612628347811771111042012460898191436"
    );

    // constants that are used to perform scalar decomposition
    // This is a matrix which is practically the LLL reduced bases
    // N = Matrix(
    // [[113482231691339203864511368254957623327,
    // 10741319382058138887739339959866629956],
    // [21482638764116277775478679919733259912,
    // -113482231691339203864511368254957623327]])

    const COEFF_N11: Self::ScalarField =
        field_new!(Fr, "113482231691339203864511368254957623327");

    const COEFF_N12: Self::ScalarField =
        field_new!(Fr, "10741319382058138887739339959866629956");

    const COEFF_N21: Self::ScalarField =
        field_new!(Fr, "21482638764116277775478679919733259912");

    const COEFF_N22: Self::ScalarField =
        field_new!(Fr, "-113482231691339203864511368254957623327");

    /// Mapping a point G to phi(G):= lambda G where phi is the endomorphism
    fn endomorphism(base: &Self::CurveAffine) -> Self::CurveAffine {
        let mut x = base.x;
        let mut y = base.y;
        let mut z = y;

        // z = y;
        let fy = Self::COEFF_A1 * (y + Self::COEFF_A2) * (y + Self::COEFF_A3);
        let gy = Self::COEFF_B1 * (y + Self::COEFF_B2) * (y + Self::COEFF_B3);
        let hy = (y + Self::COEFF_C1) * (y + Self::COEFF_C2);

        x = x * fy * hy;
        y = gy * z;
        z = hy * z;

        Self::CurveProjective::new(x, y, Fq::one(), z).into_affine()
    }

    /// Decompose a scalar s into k1, k2, s.t. s = k1 + lambda k2
    /// via a Babai's nearest plane algorithm.
    fn scalar_decomposition(
        scalar: &Self::ScalarField,
    ) -> (Self::ScalarField, Self::ScalarField) {
        let tmp: BigInteger256 = (*scalar).into();
        let scalar_z: BigUint = tmp.into();

        let tmp: BigInteger256 = Self::COEFF_N11.into();
        let n11: BigUint = tmp.into();

        let tmp: BigInteger256 = Self::COEFF_N12.into();
        let n12: BigUint = tmp.into();

        let r: BigUint = <FrParameters as FpParameters>::MODULUS.into();

        // beta = vector([n,0]) * self.curve.N_inv
        let beta_1 = scalar_z.clone() * n11;
        let beta_2 = scalar_z * n12;

        let beta_1 = beta_1 / r.clone();
        let beta_2 = beta_2 / r;

        // b = vector([int(beta[0]), int(beta[1])]) * self.curve.N
        let beta_1 = Fr::from(beta_1);
        let beta_2 = Fr::from(beta_2);
        let b1 = beta_1 * Self::COEFF_N11 + beta_2 * Self::COEFF_N21;
        let b2 = beta_1 * Self::COEFF_N12 + beta_2 * Self::COEFF_N22;

        let k1 = (*scalar) - b1;
        let k2 = -b2;
        (k1, k2)
    }

    /// perform GLV multiplication
    fn glv_mul(
        base: &Self::CurveAffine,
        scalar: &Self::ScalarField,
    ) -> Self::CurveProjective {
        let psi_base = Self::endomorphism(&base);
        let (k1, k2) = Self::scalar_decomposition(scalar);
        multi_scalar_mul(&base, &k1, &psi_base, &k2)
    }
}

// Here we need to implement a customized MSM algorithm, since we know that
// the high bits of Fr are restricted to be small, i.e. ~ 128 bits.
// This MSM will save us some 128 doublings.
pub fn multi_scalar_mul(
    base: &crate::EdwardsAffine,
    scalar_1: &Fr,
    endor_base: &crate::EdwardsAffine,
    scalar_2: &Fr,
) -> crate::EdwardsProjective {
    let mut b1 = (*base).into_projective();
    let mut s1 = *scalar_1;
    let mut b2 = (*endor_base).into_projective();
    let mut s2 = *scalar_2;

    let r_over_2: Fr =
        <FrParameters as FpParameters>::MODULUS_MINUS_ONE_DIV_TWO.into();

    if s1 > r_over_2 {
        b1 = -b1;
        s1 = -s1;
    }
    if s2 > r_over_2 {
        b2 = -b2;
        s2 = -s2;
    }
    let s1: BigInteger256 = s1.into();
    let s2: BigInteger256 = s2.into();

    let b1b2 = b1 + b2;

    let s1_bits = s1.to_bits_le();
    let s2_bits = s2.to_bits_le();
    let s1_len = get_bits(&s1_bits);
    let s2_len = get_bits(&s2_bits);
    let len = max(s1_len, s2_len) as usize;

    let mut res = crate::EdwardsProjective::zero();
    for i in 0..len {
        res = res.double();
        if s1_bits[len - i - 1] && !s2_bits[len - i - 1] {
            res += b1
        }
        if !s1_bits[len - i - 1] && s2_bits[len - i - 1] {
            res += b2
        }
        if s1_bits[len - i - 1] && s2_bits[len - i - 1] {
            res += b1b2
        }
    }
    res
}

/// return the highest non-zero bits of a bit string.
fn get_bits(a: &[bool]) -> u16 {
    let mut res = 256;
    for e in a.iter().rev() {
        if !e {
            res -= 1;
        } else {
            return res;
        }
    }
    res
}
