#!/usr/bin/env python

import math
import optparse
import json

parser = optparse.OptionParser(description="""
""")
parser.add_option('--radius', dest='radius', default=5,
                    action='store', type='float',
                    help='radius of cylinder. default = 5')
parser.add_option('--leds', dest='leds', default=150,
                    action='store', type='int',
                    help='height of cylinder.  default = 150')
parser.add_option('--poles', dest='poles', default=20,
                    action='store', type='int',
                    help='number of poles.  default = 20')
options, args = parser.parse_args()

P = options.poles
N = options.leds
R = options.radius
leds = []
for p in range(P):
    for i in range(N):
        polar = (math.pi/2)*i/N 
        azimouth = (2*math.pi)*p/P
        x = R*math.sin(polar)*math.cos(azimouth)
        y = R*math.sin(polar)*math.sin(azimouth)
        z = R*math.cos(polar)
        leds += [{"point" : [x, y, z]}]

print json.dumps(leds)