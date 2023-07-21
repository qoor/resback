use rand::seq::SliceRandom;

use super::UserType;

const USER_PICTURE_BASE_URL: &str =
    "https://respec-public.s3.ap-northeast-2.amazonaws.com/official-profile-image";
const NORMAL_USER_PICTURE_PATH: &str = "/normal";
const SENIOR_USER_PICTURE_PATH: &str = "/senior";

const NORMAL_USER_PICTURE_FILE: &[&str] = &[
    "/00001-2311288384.png",
    "/00001-4019179572.png",
    "/00003-2669798765.png",
    "/00003-2996407054.png",
    "/00004-1974761384.png",
    "/00004-2873347302.png",
    "/00004-791457417.png",
    "/00005-2421157642.png",
    "/00005-3582660123.png",
    "/00006-4108556296.png",
    "/00007-2393361731.png",
    "/00007-763610471.png",
    "/00008-937165181.png",
    "/00009-2318225659.png",
    "/00009-746171883.png",
    "/00010-684132780.png",
    "/00011-1805768178.png",
    "/00011-2934834605.png",
    "/00013-2189777024.png",
    "/00013-2241343660.png",
    "/00015-1644000371.png",
    "/00017-2694564199.png",
    "/00018-2037563641.png",
    "/00019-4187890009.png",
    "/00020-1586693095.png",
    "/00020-598865086.png",
    "/00021-1383895046.png",
    "/00021-1604167167.png",
    "/00023-3817656826.png",
    "/00025-1552547580.png",
    "/00030-3384597830.png",
    "/00035-3850697136.png",
    "/00036-2693265961.png",
    "/00038-2431092388.png",
    "/00040-132639853.png",
    "/00042-307429062.png",
    "/00044-1318476520.png",
    "/00046-2396405431.png",
];

const SENIOR_USER_PICTURE_FILE: &[&str] = &[
    "/00001-3434581533.png",
    "/00004-3977531429.png",
    "/00005-1830399694.png",
    "/00006-1740059079.png",
    "/00007-1740059080.png",
    "/00007-2124513801.png",
    "/00008-1740059081.png",
    "/00008-4103619087.png",
    "/00009-1740059082.png",
    "/00010-1740059083.png",
    "/00012-1740059085.png",
    "/00012-2940656816.png",
    "/00013-1740059086.png",
    "/00013-2119529543.png",
    "/00014-1740059087.png",
    "/00014-2109521521.png",
    "/00015-206971507.png",
    "/00016-1740059089.png",
    "/00016-3674672569.png",
    "/00017-1740059090.png",
    "/00018-3822707.png",
    "/00018-493990658.png",
    "/00019-4003472170.png",
    "/00020-493990660.png",
    "/00020-626368641.png",
    "/00021-493990661.png",
    "/00023-493990663.png",
    "/00024-4175073903.png",
    "/00024-493990664.png",
    "/00026-3787837631.png",
    "/00026-493990666.png",
    "/00029-493990669.png",
    "/00030-354789544.png",
    "/00030-493990670.png",
    "/00031-493990671.png",
    "/00034-861878526.png",
    "/00035-4148510545.png",
    "/00037-4196540852.png",
];

pub fn get_random_user_picture_url(user_type: UserType) -> String {
    let mut url = String::from(USER_PICTURE_BASE_URL);

    match user_type {
        UserType::NormalUser => {
            url += &(NORMAL_USER_PICTURE_PATH.to_string()
                + NORMAL_USER_PICTURE_FILE.choose(&mut rand::thread_rng()).unwrap())
        }
        UserType::SeniorUser => {
            url += &(SENIOR_USER_PICTURE_PATH.to_string()
                + SENIOR_USER_PICTURE_FILE.choose(&mut rand::thread_rng()).unwrap())
        }
    };

    url
}
