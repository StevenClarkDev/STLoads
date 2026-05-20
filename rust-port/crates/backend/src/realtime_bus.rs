use shared::RealtimeEvent;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RealtimeSubscriptionScope {
    pub tenant_ids: Vec<String>,
    pub is_platform_admin: bool,
    pub requested_tenant_id: Option<String>,
    pub requested_resource_type: Option<String>,
    pub requested_resource_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RoutedRealtimeEvent {
    pub payload: RealtimeEvent,
    pub target_user_ids: Vec<u64>,
    pub target_role_keys: Vec<String>,
    pub target_permission_keys: Vec<String>,
    pub topics: Vec<String>,
    pub tenant_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<u64>,
}

impl RoutedRealtimeEvent {
    pub fn new(payload: RealtimeEvent) -> Self {
        Self {
            payload,
            target_user_ids: Vec::new(),
            target_role_keys: Vec::new(),
            target_permission_keys: Vec::new(),
            topics: Vec::new(),
            tenant_id: None,
            resource_type: None,
            resource_id: None,
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

    pub fn for_tenant<S>(mut self, tenant_id: S) -> Self
    where
        S: Into<String>,
    {
        let tenant_id = tenant_id.into();
        if !tenant_id.trim().is_empty() {
            self.tenant_id = Some(tenant_id);
        }
        self
    }

    pub fn for_resource<S>(mut self, resource_type: S, resource_id: u64) -> Self
    where
        S: Into<String>,
    {
        let resource_type = resource_type.into();
        if !resource_type.trim().is_empty() && resource_id > 0 {
            self.resource_type = Some(resource_type);
            self.resource_id = Some(resource_id);
        }
        self
    }

    pub fn should_deliver_to(
        &self,
        user_id: u64,
        role_key: &str,
        permission_keys: &[String],
        requested_topics: &[String],
        subscription_scope: &RealtimeSubscriptionScope,
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

        if !self.matches_subscription_scope(subscription_scope) {
            return false;
        }

        requested_topics.is_empty()
            || self.topics.is_empty()
            || requested_topics
                .iter()
                .any(|topic| self.topics.iter().any(|candidate| candidate == topic))
    }

    fn matches_subscription_scope(&self, scope: &RealtimeSubscriptionScope) -> bool {
        if let Some(event_tenant_id) = self.tenant_id.as_deref() {
            if let Some(requested_tenant_id) = scope.requested_tenant_id.as_deref() {
                if requested_tenant_id != event_tenant_id {
                    return false;
                }
            }

            if !scope.is_platform_admin
                && !scope
                    .tenant_ids
                    .iter()
                    .any(|tenant_id| tenant_id == event_tenant_id)
            {
                return false;
            }
        }

        if let Some(requested_resource_type) = scope.requested_resource_type.as_deref() {
            if self.resource_type.as_deref() != Some(requested_resource_type) {
                return false;
            }
        }

        if let Some(requested_resource_id) = scope.requested_resource_id {
            if self.resource_id != Some(requested_resource_id) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use shared::{RealtimeEvent, RealtimeEventKind};

    use super::{RealtimeSubscriptionScope, RoutedRealtimeEvent};

    fn event() -> RealtimeEvent {
        RealtimeEvent {
            kind: RealtimeEventKind::LoadBoardListingUpdated,
            leg_id: Some(42),
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(7),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: "Listing changed.".into(),
        }
    }

    #[test]
    fn tenant_and_resource_scope_blocks_cross_tenant_subscription() {
        let routed = RoutedRealtimeEvent::new(event())
            .for_role_keys(["carrier"])
            .for_tenant("tenant-a")
            .for_resource("load_leg", 42)
            .with_topics(["load_board"]);
        let permissions = Vec::<String>::new();
        let wrong_tenant = RealtimeSubscriptionScope {
            tenant_ids: vec!["tenant-b".into()],
            requested_tenant_id: Some("tenant-b".into()),
            requested_resource_type: Some("load_leg".into()),
            requested_resource_id: Some(42),
            ..Default::default()
        };
        let right_tenant_wrong_resource = RealtimeSubscriptionScope {
            tenant_ids: vec!["tenant-a".into()],
            requested_tenant_id: Some("tenant-a".into()),
            requested_resource_type: Some("load_leg".into()),
            requested_resource_id: Some(99),
            ..Default::default()
        };
        let right_scope = RealtimeSubscriptionScope {
            tenant_ids: vec!["tenant-a".into()],
            requested_tenant_id: Some("tenant-a".into()),
            requested_resource_type: Some("load_leg".into()),
            requested_resource_id: Some(42),
            ..Default::default()
        };

        assert!(!routed.should_deliver_to(
            7,
            "carrier",
            &permissions,
            &["load_board".into()],
            &wrong_tenant
        ));
        assert!(!routed.should_deliver_to(
            7,
            "carrier",
            &permissions,
            &["load_board".into()],
            &right_tenant_wrong_resource
        ));
        assert!(routed.should_deliver_to(
            7,
            "carrier",
            &permissions,
            &["load_board".into()],
            &right_scope
        ));
    }
}
