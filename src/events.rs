use crate::audio::AudioEvent;
use crate::enemies::{FvzEvent, RollbackSafeEvents, SafeEvent};
use crate::GameState;
use bevy::prelude::*;
use bevy::utils::HashMap;

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SafeEventsCache>()
            .add_system_set(SystemSet::on_update(GameState::InGame).with_system(propagate));
    }
}

#[derive(Default)]
pub struct SafeEventsCache(HashMap<u32, SafeEvent>);

pub fn propagate(
    mut events_cache: ResMut<SafeEventsCache>,
    mut rollback_safe_events: ResMut<RollbackSafeEvents>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in rollback_safe_events.0.drain(..) {
        if events_cache.0.contains_key(&event.id) {
            continue;
        }
        match event.event {
            FvzEvent::EnemyFall => audio_events.send(AudioEvent::EnemyFall),
            FvzEvent::PlayerHit => audio_events.send(AudioEvent::PlayerHit),
            FvzEvent::PlayerHitBullet => audio_events.send(AudioEvent::PlayerHitBullet),
            FvzEvent::Lost => audio_events.send(AudioEvent::Lost),
            FvzEvent::Pew => audio_events.send(AudioEvent::Pew),
            FvzEvent::Revive => audio_events.send(AudioEvent::Revive),
        }
    }
    events_cache
        .0
        .iter_mut()
        .for_each(|(_, event)| event.real_age += 1);
    events_cache.0.retain(|_, event| event.real_age < 60);
}
