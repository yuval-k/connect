extern crate gcc;

#[cfg(feature = "ledscape")]
fn build() {
gcc::Config::new()
                .file("./lib/LEDscape/pru.c")
                .file("./lib/LEDscape/ledscape.c")
                .file("./lib/LEDscape/am335x/app_loader/interface/prussdrv.c")
                .include("./lib/LEDscape/am335x/app_loader/include/")
                .compile("libledscape.a");
}

#[cfg(not(feature = "ledscape"))]
fn build() {
}

fn main() {
    build();
}