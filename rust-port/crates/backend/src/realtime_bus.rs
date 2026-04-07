use shared::RealtimeEvent;

#[derive(Debug, Clone)]
pub struct RoutedRealtimeEvent {
    pub payload: RealtimeEvent,
    pub target_user_ids: Vec<u64>,
    pub target_role_keys: Vec<String>,
    pub target_permission_keys: Vec<String>,
    pub topics: Vec<String>,
}

impl RoutedRealtimeEvent {
    pub fn new(payload: RealtimeEvent) -> Self {
        Self {
            payload,
            target_user_ids: Vec::new(),
            target_role_keys: Vec::new(),
            target_permission_keys: Vec::new(),
            topics: Vec::new(),
        }
    }

    pub fn for_user_ids<I>(mut self, user_ids: I) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        for user_id in user_ids {
            if user_id == 0 || self.target_user_ids.contains(&user_id) {
                continue;
            }

            self.target_user_ids.push(user_id);
        }

        self
    }

    pub fn for_role_keys<I, S>(mut self, role_keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for role_key in role_keys {
            let role_key = role_key.into();
            if role_key.trim().is_empty() || self.target_role_keys.contains(&role_key) {
                continue;
            }

            self.target_role_keys.push(role_key);
        }

        self
    }

    pub fn for_permission_keys<I, S>(mut self, permission_keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for permission_key in permission_keys {
            let permission_key = permission_key.into();
            if permission_key.trim().is_empty()
                || self.target_permission_keys.contains(&permission_key)
            {
                continue;
            }

            self.target_permission_keys.push(permission_key);
        }

        self
    }

    pub fn with_topics<I, S>(mut self, topics: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for topic in topics {
            let topic = topic.into();
            if topic.trim().is_empty() || self.topics.contains(&topic) {
                continue;
            }

            self.topics.push(topic);
        }

        self
    }

    pub fn should_deliver_to(
        &self,
        user_id: u64,
        role_key: &str,
        permission_keys: &[String],
        requested_topics: &[String],
    ) -> bool {
        let is_targeted = self.target_user_ids.contains(&user_id)
            || self
                .target_role_keys
                .iter()
                .any(|candidate| candidate == role_key)
            || permission_keys.iter().any(|permission| {
                self.target_permission_keys
                    .iter()
                    .any(|candidate| candidate == permission)
            });

        if !is_targeted {
            return false;
        }

        requested_topics.is_empty()
            || self.topics.is_empty()
            || requested_topics
                .iter()
                .any(|topic| self.topics.iter().any(|candidate| candidate == topic))
    }
}
