# -*- coding: utf-8 -*-
"""
Created on Mon Aug 29 16:13:00 2022

@author: Till
@author: Leonard
"""

import json
import subprocess
import os
import  sys


# First create Filter

file = open("filter.txt", "w")

with open('edge_type_config.json', 'r') as f:
    data = json.loads(f.read())    
for tagMap in data.values():
    for edgeType in tagMap:
        for tag in edgeType["tags"] :
            file.write("w/{}={}\n".format(tag["key"], tag["value"]));
            
with open('sights_config.json', 'r') as f:
    data = json.loads(f.read())    
for tagMap in data.values():
    for category in tagMap:
        for tag in category["tags"] :
            file.write("n/{}={}\n".format(tag["key"], tag["value"]));

file.close()



# Apply Filtering with osmium

source =  str(sys.argv[1])
name = os.path.basename(source)

docker = "/var/lib/docker/volumes/enprofmi2022_data/_data/osm_graphs"

#Only supposed to run on unix shells
cmd = f"osmium tags-filter {source} --overwrite --expressions filter.txt -o {docker}/{name}"
a = subprocess.run([cmd], shell=True) 
