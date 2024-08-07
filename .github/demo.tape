# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ./examples/sparkline.tape`
Output ".github/preview.gif"
Output ".github/preview.mp4"
Set FontSize 14
Set Theme "OceanicMaterial"
Set Width 1920
Set Height 1080
Hide
Type "nvim .github/document.py"
Enter
Sleep 2s
Show
# select first
Type "6"
Show
Sleep 0.3s
Show
Sleep 0.3s
Type j
Show
Sleep 0.3s
Type w
Sleep 0.3s
Show
Sleep 0.3s
Type w
Sleep 0.3s
Show

# function doc string
Space
Type f
Type a
Type U
Sleep 0.3s
Show
Type p
Sleep 0.3s
Show
Type F
Sleep 0.3s
Show
Enter
Sleep 7s

Type "g"
Type "g"
Sleep 0.3s
Show

# class doc string
Space
Type f
Type a
Type U
Sleep 0.3s
Show
Type p
Sleep 0.3s
Show
Type C
Sleep 0.3s
Show
Type D
Sleep 0.3s
Show
Enter
Sleep 7s

Type "/"
Sleep 0.3s
Show
Type "d"
Sleep 0.3s
Show
Type "e"
Sleep 0.3s
Show
Type "f"
Sleep 0.3s
Show
Enter
Show

# Back to function
Type "n"
Sleep 0.3s
Show
Enter
Sleep 0.3s
Ctrl+L

Space
Type f
Type a
Type U
Sleep 0.3s
Show
Type p
Sleep 0.3s
Show
Type F
Sleep 0.3s
Show
Type C
Sleep 0.3s
Enter
Sleep 7s
