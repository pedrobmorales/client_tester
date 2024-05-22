use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub user_info_id: String,
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: String,
}

pub fn get_user(id: usize) -> UserInfo {
    let email = format!("user{id:0>2}@optm.com");
    let first_name = format!("User{id:0>2}");
    let last_name = format!("Optm{id:0>2}");
    let display_name = format!("{first_name} {last_name}");

    let created_at = "2023-05-05T21:30:38.899Z".to_string();
    let user_info_ids = [
        "3273acee-9def-42f3-98b6-b9dcd2b3b8de",
        "5aba863d-67dc-4732-b731-a5fd0ec7ac4d",
        "b22a8c74-3ce7-4375-a4aa-4f70d49bb7fb",
        "8aec20cd-5d9d-4d2c-895f-5b639f933df2",
        "86d5cc7d-915d-46ae-add3-afc7c8e2ff4a",
        "e3d352ef-95b2-4229-a6e2-3292b888b906",
        "24c4056b-3bec-4fb7-a36b-64549d27895a",
        "4cd105fc-0844-4019-a07b-a3c71282b15f",
        "86d8be71-bc28-4b23-888c-dc38a289f991",
        "77bab93e-5716-4fe3-91b4-77376c8a877d",
    ];
    let index: usize = if id > 0 { id - 1 } else { 0 };

    let user_info_id = user_info_ids[index].to_string();
    UserInfo {
        user_info_id,
        email,
        display_name,
        first_name,
        last_name,
        created_at,
    }
}
