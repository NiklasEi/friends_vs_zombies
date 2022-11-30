use crate::events::propagate;
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AudioEvent>().add_system_set(
            SystemSet::on_update(GameState::InGame).with_system(enemy_falls.after(propagate)),
        );
    }
}

pub enum AudioEvent {
    EnemyFall,
    PlayerHit,
    PlayerHitBullet,
}

fn enemy_falls(mut events: EventReader<AudioEvent>, sound: Res<AudioAssets>, audio: Res<Audio>) {
    for event in events.iter() {
        match event {
            AudioEvent::EnemyFall => audio.play(sound.enemy_fall.clone()),
            AudioEvent::PlayerHit => audio.play(sound.player_hit.clone()),
            AudioEvent::PlayerHitBullet => audio.play(sound.player_hit_bullet.clone()),
        };
    }
}
