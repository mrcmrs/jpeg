use jpeg::{get, save};


fn main() {
    // let img_path = "img/white_square.jpg";
    // let img_path = "img/white_square_16x16.jpg";
    // let img_path = "img/sq16rdot.jpg";
    // let img_path = "img/rec32dot.jpg";
    let img_path = "img/maps.jpg";

    let pic = get(img_path);
    save(pic);

}

