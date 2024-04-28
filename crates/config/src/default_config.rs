pub static DEFAULT_CONFIG: &str = r##"theme = ""
mouse_scroll_lines = 3
gutter_width = 6
line_numbers = "RelativeNumbered"
background = "Dark"
empty_line_char = "~"
show_diagnostics = true

[keys.normal]
"n" = "FindNext"
"S-K" = "Hover"
"S-D" = "DeleteUntilEOL"
"S-N" = "FindPrevious"
"w" = "NextWord"
"b" = "PreviousWord"
"p" = "PasteBelow"
"a" = [{ EnterMode = "Insert" }, "MoveRight"]
"S-A" = [{ EnterMode = "Insert" }, "MoveToLineEnd", "MoveRight"]
"S-O" = ["InsertLineAbove", { EnterMode = "Insert" }]
"o" = ["InsertLineBelow", { EnterMode = "Insert" }]
"u" = "Undo"
"k" = "MoveUp"
"Up" = "MoveUp"
"j" = "MoveDown"
"h" = "MoveLeft"
"l" = "MoveRight"
"S-G" = "MoveToBottom"
"$" = "MoveToLineEnd"
"0" = "MoveToLineStart"
"x" = "DeleteCurrentChar"
"/" = { EnterMode = "Search" }
"i" = { EnterMode = "Insert" }
"S-I" = [{ EnterMode = "Insert" }, "MoveToLineStart"]
":" = { EnterMode = "Command" }
"Down" = "MoveDown"
"Left" = "MoveLeft"
"Right" = "MoveRight"
"C-d" = "PageDown"
"C-u" = "PageUp"
"C-z" = "Quit"
"End" = "MoveToLineEnd"
"Home" = "MoveToLineStart"
"g" = { "g" = "MoveToTop", "d" = "GoToDefinition" } 
"d" = { "w" = "DeleteWord", "d" = "DeleteLine", "b" = "DeleteBack" }
"z" = { "z" = "CenterLine" }

[keys.insert]
"Enter" = "InsertLine"
"Backspace" = "DeletePreviousChar"
"Tab" = "InsertTab"
"Esc" = { EnterMode = "Normal" }
"C-c" = { EnterMode = "Normal" }

[keys.command]
"Esc" = { EnterMode = "Normal" }
"C-c" = { EnterMode = "Normal" }
"Enter" = "ExecuteCommand"
"Backspace" = "DeletePreviousChar"
"##;
