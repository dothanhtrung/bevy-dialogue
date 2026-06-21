<div align="center">

bevy_dialogue
=============

[![crates.io](https://img.shields.io/crates/v/bevy_dialogue)](https://crates.io/crates/bevy_dialogue)
[![docs.rs](https://docs.rs/bevy_dialogue/badge.svg)](https://docs.rs/bevy_dialogue)
[![dependency status](https://deps.rs/crate/bevy_dialogue/latest/status.svg)](https://deps.rs/crate/bevy_dialogue/latest)
[![pipeline status](https://gitlab.com/245project/bevy-plugin/bevy-dialogue/badges/master/pipeline.svg)](https://gitlab.com/245project/bevy-plugin/bevy-dialogue/-/commits/master)

[![Gitlab](https://img.shields.io/badge/gitlab-%23181717.svg?style=for-the-badge&logo=gitlab&logoColor=white)](https://gitlab.com/245project/bevy-plugin/bevy-dialogue)
[![Github](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/dothanhtrung/bevy-dialogue)

</div>

Bevy plugin for load and retrieve characters dialogues.

The dialogues can be create and edit with GUI tool [Dialogue Editor](https://github.com/dothanhtrung/dialogue-editor)

Asset Syntax
------------

```ron
(
    dialogues: {
        <class_id>: {
            <state_id>: [
                (
                    contents: {
                        "<language code>": "Dialogue content. {{variable}} is supported",
                    },
                    affects: {
                        <target_class_id>: <target_state_id>,
                    },
                ),
            ],
        },
    },
)
```

* `class_id`: `u64`. The character class id. For example: Villager, Hero, etc. should have unique id.
* `state_id`: `u64`. The character state id. For example: Idle, Arguing, Cheering, etc. should have unique id.
* `language_code`: `String`. 3 character language code by ISO 639-3. For example: `eng`, `spa`, etc.
* `affects`: This mean the state of entity with `target_class_id` will be change to `target_state_id` after this dialog.


Dialogue asset can be text file (RON format) or binary file. It is recommended to use
[Dialogue Editor](https://github.com/dothanhtrung/dialogue-editor) for creating and exporting your
dialogue asset.

Quickstart
----------

Please see [examples](./examples).

License
-------

Please see [LICENSE](./LICENSE).


Compatible Bevy Versions
------------------------

| bevy | bevy_dialogue |
|------|---------------|
| 0.19 | 0.2           |
| 0.18 | 0.1           |

---------

<div align="center">

![git_bevy-dialogue](https://count.getloli.com/@git_bevy-dialogue?name=git_bevy-dialogue&theme=random&padding=10&offset=0&align=top&scale=1&pixelated=1&darkmode=auto)

</div>
