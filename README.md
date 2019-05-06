# multiline-console
Quick, no BS implementation of a multiline console prompt. The crate wraps around the `console` crate.

## Features
* Arrow navigation – `up, down, left, right`.
* `Enter` to create a new line or push the contents beyond the cursor into a new line.
* `Enter` on an empty last line in order to submit the input.
* `Backspace` at the beginning of a line to merge two lines.
* Basic prompt printing.

## TODO
* Windows is still icky
* Separate `src/console_patch` and make a PR and make it into a patch.