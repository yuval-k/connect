                        
[5:11 PM, 4/14/2017] איתי רונן: ער?                        
[6:59 PM, 4/14/2017] איתי רונן: אלגוריתמיקה של אנימציה:

אמביאנט - רקע כחול. ניצנוץ כוכבים, טבעת קונצנטרית מידי פעם בגווני כחול

מגע בעמוד - זרימה כלפי מעלה של צבע. מבעבע בגוונים.

שרשרת - זרימה כלפי מעלה בשני העמודים. ערבוב צבעים.
כשהצבעים נפגשים במרכז המעגל הם זורמים חזרה לעמודים בצבעים המעורבבים ובנוסף יורדות טבעות קונצנטריות בצבעים המעורבבים.                        
[7:18 PM, 4/14/2017] איתי רונן: יש לך הגדרות לעוד אנימציות? נראה לי שנגדיר כמה אנימציות בסיסיות ואנחנו מסודרים



Sikum:

BBB will get messages (which needs to be timed out) via WiFI or some other MCI (perhaps via serial or spi?!).
regardless the other logic that it might need to do is to sync all the nodes via a leg.


# Prepare the BBB:
// disable HDMI:
// 

Generate pru binaries:
install node: apt-get install nodejs-legacy
install cross gcc: gcc-arm-linux-gnueabihf OR gcc-arm-linux-gnueabi

make CROSS_COMPILE=arm-linux-gnueabihf- 

generate library:

(fix make file and add override to CFLAGS)
make CROSS_COMPILE=arm-linux-gnueabihf- CFLAGS=-fPIC libledscape.a

(fix make file and add override to C_FLAGS)
make CROSS_COMPILE=arm-linux-gnueabihf- C_FLAGS=-fPIC -C ./am335x/app_loader/interface/

disable hdmi (latest firmware) - uncomment:

##BeagleBone Black: HDMI (Audio/Video) disabled:
dtb=am335x-boneblack-emmc-overlay.dtb


verify:
sudo cat /sys/devices/platform/bone_capemgr/slots



# upgrade to latest jessie with latest kernel
/opt/tools/scripts/upgrade_kernel

# load module
(see: http://elinux.org/EBC_Exercise_30_PRU_via_remoteproc_and_RPMsg)
#enable uio in
cd /opt/source/dtb-4.4-ti

edit:  src/arm/am335x-boneblack-wireless-emmc-overlay.dts
make sure that /boot/uEnv.txt has dtb set to m335x-boneblack-wireless-emmc-overlay.dtb:
dtb=am335x-boneblack-wireless-emmc-overlay.dtb

comment out #include "am33xx-pruss-rproc.dtsi"
uncomment #include "am33xx-pruss-uio.dtsi"
add blacklist;
cat /etc/modprobe.d/pruss-blacklist.conf

blacklist pruss
blacklist pruss_intc
blacklist pru-rproc

and then make && sudo make install

sudo modprobe uio_pruss

###########################################################

for 5 seconds
color clibs to 30% from the pole. 
brighter color climbs out.

when two people touch a pole:


colors contrinue to climb. when they reach to the top after about more 10 seconds
when they meet, the color is combined the goes down the poles.
when the color goes down a circle pattern emerges with clear


state:
touched, connected, untoched

if is from {*} -> connected:
    if both poles are tocued / none:
        get the touched turn the touched animations 
    if one of the pole is connected => nothing

if connected(x) -> connected(y) // tri angle use case
    => nothing


each pole has a level, each animation has (undo animation) so it can transition smoothly

undo / regular animations can be immediatly replaced, and can be created from each outher

following transitions allow:
nothing -> toched
nothing -> connect
touched -> connect
touched -> reversed touch
reversed touch -> nothing
connect -> reverse connect -> reverse touch -> nothing
connect -> explode 
explode -> reverse explode

for each pole, remove current state, unless it is connected and the connection is still valid.
loop and assign state.

connected
 connected1(is other full)
 connected2(is other full)
 when both connected are finished



 riser on (only when chain)


 riser off; explosion on;regular touch off; ceiling touch on;

 5 rise notes;


TOUCH ON
touch off
ceiling touch on

