#!/usr/bin/env python3
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
current_dir = os.getcwd()

os.chdir("backoffice/asset/embed_hidden/js")
os.symlink("../../embed/js", "assets", target_is_directory=True)

os.chdir(current_dir)

os.chdir("public/asset/embed_hidden/js")
os.symlink("../../embed/js", "assets", target_is_directory=True)
