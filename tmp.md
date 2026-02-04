
### sub issue 3
Currently the title is hardcoded to appear at the top of the stat list:
```
Pacman v7.1.0 - libalpm v16.0.1
-------------------------------
```
The user should be able to configure this from the stats display options like so:
```
stats = [
    "title",
    "installed",

output:

Pacman v7.1.0 - libalpm v16.0.1
-------------------------------
Installed: 1268
```

Another option should be added to stats: "separator" that prints to be the same length as title, but without the text
```
stats = [
    "separator",
    "installed",


output:

-------------------------------
Installed: 1268
```

Implementation notes: 
- Add title and separator to StatId enum in stats.rs
- update StatId::label() - should return empty string as it is printed dynamically in ui/mod.rs
- similarly, update StatId::format_value(), returns None
- The meat of this ticket will be refactoring the display loop in ui/mod.rs -> display_stats_with_graphics(). 
- Lastly, update default_stats() in config.rs with StatId::Title at the start of the default array



### sub issue 1
The text of the title should be configurable with the default being the system pacman/libalpm version. There should also be other built in options like displaying just the pacman version without libalpm, or the version of pacfetch.
```
[display.title]
text = "custom title" // default, pacman_ver, pacfetch_ver

custom title   
------------


[display.title]
text = "default" // default, pacman_ver, pacfetch_ver

Pacman v7.1.0 - libalpm v16.0.1
-------------------------------


[display.title]
text = "pacman_ver" // default, pacman_ver, pacfetch_ver

Pacman v7.1.0 
-------------
```







Improve customization options for title:
```
output:

Pacman v7.1.0 - libalpm v16.0.1
-------------------------------
```

### Enhancement 1: 
The text of the title should be configurable with the default being the system pacman/libalpm version. There should also be other built in options like using just the pacman version without libalpm, or the version of pacfetch. 
```
[display.title]
text = "custom title" // default, pacman_ver, pacfetch_ver
```
### Enhancement 2:
 The color of the title and it's underline should be configurable
```
[display.title]
text_color = "blue"
line_color = "white"
```

### Enhancement 3: 
Title should be treated as a positionable/customizable field rather than being hardcoded
```
stats = [
    "title",
    "installed",
    "upgradable",
```

### Enhancement 4: 
The length of the title underline should be configurable to the length of:
- The title
- The longest line in the output of the labels and stats
- A custom value

```
[display.title]
width = "title" // or "content" or 40, etc.


   width = "title"

Pacman v7.1.0 - libalpm v16.0.1
-------------------------------


   width = "content"

         Pacman v7.1.0 - libalpm v16.0.1
------------------------------------------------
Mirror URL: https://mirrors.kernel.org/archlinux


   width = 40

    Pacman v7.1.0 - libalpm v16.0.1
----------------------------------------
```
### Enhancement 5: 
new title styles, embedded title placement, and customizable line characters:
```
pacfetch.toml:

  [display.title.top]
  style = "embedded"         #  "stacked" or "embedded" with stacked being the default
  line = "─"
  left_cap = "╭"
  right_cap = "╮"

Output:
    
  ╭────────── Pacman v7.1.0 ───────────╮

```



### Enhancement 6:
 we should have new title types: title_top, title_middle, title_bottom that correspond to these types of outputs:
```
stats = [
    "title_top",
    "installed",
    "upgradable",
    "title_middle",
    "orphaned_packages",
    "package_cache",
    "title_bottom",
]

  [display.title.top] 
  text = "default"
  width = 40
  style = "stacked"       
  line = "─"
  left_cap = ""
  right_cap = ""

  [display.title.middle]  
  text = ""
  width = 40
  style = "embedded"      
  line = "-"
  left_cap = "|"
  right_cap = "|"

  [display.title.bottom`]
  text = "custom text"
  value = "default"
  width = 40
  style = "embedded"       
  line = "─"
  left_cap = "╯"
  right_cap = "╯"


Output:
         Pacman v7.1.0 - libalpm v16.0.1
-------------------------------------------------
    Installed: 1268
    Upgradable: 4
├───────────────────────────────────────────────┤
    Package Cache: 0.00 MiB
    Orphaned Packages: 12 (148.81 MiB)
╰─────────────────custom text───────────────────╯
```
