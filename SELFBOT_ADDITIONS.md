# Documentation of additions for selfbot use

-   ## Edit Group

    PATCH /api/vX/channels/CHANNEL_ID

    | field | value                |
    | ----- | -------------------- |
    | icon  | base64 encoded image |
    | name  | string               |

    Response is the new group data:

    ```json
    {
        "icon": "ICON_ID",
        "id": "CHANNEL_ID",
        "last_message_id": "LAST_MESSAGE_ID",
        "name": "GROUP_NAME",
        "owner_id": "OWNER_ID",
        "recipients": [
            {
                "id": "720613853431463977",
                "username": "other_user",
                "avatar": null,
                "discriminator": "8286",
                "public_flags": 0
            },
            ...
        ],
        "type": 3
    }
    ```

    ## Add Group Recipient

    adds a user to the recipients of a group

    PUT /api/vX/channels/CHANNEL_ID/recipients/USER_ID

    PREREQUISITES: you have to be friends with them
