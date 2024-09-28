use self::{
    building::gen_shader_wgsl,
    calculations::{CalculationInfo, CalculationStructures},
    lines::Lines,
};
use crate::{
    components::{buffer::ShaderInput, extract::SdfBindings},
    flag::{CompFlag, FlagStorage, NewSdfFlags, SdfFlags},
    operations::OperationInfos,
    ComdfPostUpdateSet,
};
use bevy::{
    prelude::*,
    render::{render_resource::Shader, Extract, RenderApp},
};
use itertools::Itertools;

pub mod bindgroups;
mod building;
pub mod calculations;
pub mod lines;

pub fn plugin(app: &mut App) {
    app.register_type::<ShaderCodeCollection>()
        .init_resource::<ShaderCodeCollection>()
        .add_plugins(calculations::plugin)
        .add_systems(
            PostUpdate,
            build_new_shaders.in_set(ComdfPostUpdateSet::BuildShaders),
        )
        .add_event::<NewShader>();

    app.sub_app_mut(RenderApp)
        .add_event::<NewShader>()
        .add_systems(ExtractSchedule, extract_new_shader_events);
}

#[derive(Debug, Default)]
pub struct CompShaderInfo {
    pub inputs: Vec<ShaderInput>,
    pub snippets: Lines,
    pub calculations: Vec<CalculationInfo>,
}

pub type CompShaderInfos = FlagStorage<CompShaderInfo, 64>;

impl CompShaderInfos {
    fn gather<'a, T, FN: Fn(&'a CompShaderInfo) -> T + 'a>(
        &'a self,
        flag: &'a CompFlag,
        func: FN,
    ) -> impl Iterator<Item = T> + 'a {
        flag.ones().map(|i| &self[i]).map(func)
    }
}

#[derive(Resource, Reflect, Deref, DerefMut, Default)]
#[reflect(Resource)]
struct ShaderCodeCollection(String);

fn build_new_shaders(
    mut events: EventReader<NewSdfFlags>,
    calc_structures: Res<CalculationStructures>,
    comp_infos: Res<CompShaderInfos>,
    op_infos: Res<OperationInfos>,
    bindings: Res<SdfBindings>,
    mut shaders: ResMut<Assets<Shader>>,
    mut new_shaders: EventWriter<NewShader>,
    mut collection: ResMut<ShaderCodeCollection>,
) {
    for new in events.read() {
        let comps = new.iter_unique_comps().collect_vec();
        let Some(bindings) = comps
            .iter()
            .map(|&flag| bindings.get(flag).copied())
            .collect::<Option<Vec<_>>>()
        else {
            error!("Flag not registered in SdfBindings: {:?}", new);
            continue;
        };

        trace!(
            "Generating shader: flags={:?}, bindings={:?}",
            new.0,
            bindings
        );

        let shader_wgsl = gen_shader_wgsl(
            new,
            comps,
            &bindings,
            &comp_infos,
            &op_infos,
            &calc_structures,
        )
        .into_file_str();
        collection.0.clone_from(&shader_wgsl);

        let shader = Shader::from_wgsl(
            shader_wgsl,
            format!("Generated in {} for flags {:?}", file!(), new),
        );
        let handle = shaders.add(shader);

        new_shaders.send(NewShader {
            flags: new.0.clone(),
            shader: handle,
            bindings,
        });
    }
}

#[derive(Event, Clone)]
pub struct NewShader {
    pub flags: SdfFlags,
    pub shader: Handle<Shader>,
    pub bindings: Vec<usize>,
}

fn extract_new_shader_events(
    mut main: Extract<EventReader<NewShader>>,
    mut render: EventWriter<NewShader>,
) {
    main.read().cloned().for_each(|new| {
        render.send(new);
    });
}
