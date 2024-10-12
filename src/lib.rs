pub use bevy::prelude::*;
use bevy::{
    render::{
        mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology
    }, sprite::Mesh2dHandle, utils::HashMap
};

mod gpu;
mod cpu;

pub struct BirdBoxesPlugin;
impl Plugin for BirdBoxesPlugin{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkSize>()
            .init_resource::<ChunkIsoReselution>()
            .add_systems(PreUpdate, add_mesh)
            .add_systems(Update, update_mesh);
    }
}

#[derive(Resource, Debug)]
pub struct ChunkSize(usize, usize);
impl ChunkSize {
    pub fn x(&self) -> usize { self.0 }
    pub fn y(&self) -> usize { self.1 }
    pub fn get(&self) -> (usize, usize){
        (self.0, self.1)
    }
}

impl Default for ChunkSize{
    fn default() -> Self{
        Self(3, 3)
    }
}

#[derive(Resource, Debug, Deref)]
pub struct ChunkIsoReselution(pub f32);

impl ChunkIsoReselution{
    pub fn get(&self) -> f32{self.0}
}
impl Default for ChunkIsoReselution{
    fn default() -> Self{
        Self(1.0)
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct IsoField(Vec<f32>);
impl IsoField {
    pub fn new(chunk_size: impl Into<(usize, usize)>) -> Self{
        let (x, y): (usize, usize) = chunk_size.into();
        let field = vec![0.0; x * y];
        Self(field)
    }
    fn index(pos: impl Into<(usize, usize)>, size: impl Into<(usize, usize)>) -> usize {
        let (x, y): (usize, usize) = pos.into();
        let (sx, _): (usize, usize) =  size.into();
        sx * y + x
    }
    pub fn get(&self, pos: impl Into<(usize, usize)>, size: impl Into<(usize, usize)>) -> f32{
        let i = Self::index(pos, size);
        self[i]
    }
    pub fn set(&mut self, size: impl Into<(usize, usize)>, pos: impl Into<(usize, usize)>, element: f32){
        let i = Self::index(pos, size);
        self[i] = element;
    }

    pub fn build_mesh(&self, size: (usize, usize), resolution: f32 ) -> Mesh{
        let mut used_indices = HashMap::<usize, usize>::new();
        let mut vertexes = Vec::new();
        let mut indices = Vec::<u32>::new();
        let mut normals = Vec::<Vec3>::new();
        let mut face_count = Vec::<usize>::new();
        let mut uvs = Vec::<Vec2>::new();
        for x in 0..(size.0 - 1){
            for y in 0..(size.1 - 1){

                let sample = [
                    self.get((x, y + 1), size), // top left
                    self.get((x + 1, y + 1), size), // top right
                    self.get((x, y), size), // bottom left
                    self.get((x + 1, y), size), // bottom right
                ]; 
                let case = get_case(sample);
                for tri_list in CASE_TABLE[case].iter(){
                    for index in tri_list.iter().filter_map(|&index| {
                        if index == -1{
                            None
                        } else {
                            Some(index as usize)
                        }
                    }){
                        for indice in reletive_indices_to_literal_indice((x, y), size, index).iter() {
                            if let Some(corrected_indice) = used_indices.get(indice){
                                indices.push(*corrected_indice as u32);
                            } else {
                                let index = indices.len();
                                used_indices.insert(*indice, index);
                                vertexes.push(new_vertex(size, *indice));
                                indices.push(index as u32);
                                normals.push(Vec3::new(0.0, 0.0, 0.0));
                                uvs.push(Vec2::new(0.0, 0.0));
                                face_count.push(0);
                            }
                        }
                    }
                } 
            }
        }

        for tri_start in (0..(indices.len())).step_by(3){
            let tri_noraml = vertexes[tri_start].normalize() + vertexes[tri_start +1].normalize() + vertexes[tri_start + 2].normalize();
            face_count[tri_start]     += 1;
            face_count[tri_start + 1] += 1;
            face_count[tri_start + 2] += 1;
        }

        for i in 0..vertexes.len(){
            normals[i] = (normals[i] / face_count[i] as f32).normalize();
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertexes)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices))
    }
}

fn reletive_indices_to_literal_indice(pos: (usize, usize), size: (usize, usize), indice: usize) -> [usize; 3]{
    todo!()
}

fn new_vertex(size: (usize, usize), index: usize) -> Vec3{
    todo!()
}

fn get_case(sample: [f32; 4]) -> usize{
    todo!()
}


#[derive(Bundle, Default)]
pub struct ChunkBundle {
    pub iso_field: IsoField,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

fn add_mesh(
    mut chunk_query: Query<(&IsoField, &Transform, Entity), Without<Mesh2dHandle>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    chunk_size: Res<ChunkSize>,
    chunk_reselution: Res<ChunkIsoReselution>,
){
    for (iso_field, pos, entity) in chunk_query.iter(){
        let mesh = iso_field.build_mesh(chunk_size.get(), chunk_reselution.get());
        let mesh_2d = Mesh2dHandle (meshes.add(mesh));
        let material = materials.add(Color::PURPLE);
        commands.entity(entity).insert((mesh_2d, material));
    }
}

fn update_mesh(
    mut chunk_query: Query<(&IsoField, &Transform, &Mesh2dHandle), Changed<IsoField>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    chunk_size: Res<ChunkSize>,
    chunk_reselution: Res<ChunkIsoReselution>,
){
    for (iso_field, pos, Mesh2dHandle(mesh_id)) in chunk_query.iter(){
        let mesh = iso_field.build_mesh(chunk_size.get(), chunk_reselution.get());
        if let Some(mut mesh_entity) = meshes.get_mut(mesh_id){
            *mesh_entity = mesh;
        } else {
            panic!("Mesh that should be there is not!")
        }
    }
}

const CASE_TABLE: [[[i8; 3]; 4]; 16] = [
    // 1
    // [0][0] 0 0 0
    // [0][0] 0 0 0
    //        0 0 0
    [[-1, -1, -1], [-1, -1, -1], [-1, -1, -1], [-1, -1, -1]],
    // 2
    // [0][0] 0 0 0
    // [1][0] \ 0 0
    //        1 \ 0
    [[0, 1, 7], [-1, -1, -1], [-1, -1, -1], [-1, -1, -1]],
    // 3
    // [1][0] 1 / 0
    // [0][0] / 0 0
    //        0 0 0
    [[1, 2, 3], [-1, -1, -1], [-1, -1, -1], [-1, -1, -1]],
    // 4
    // [0][1] 0 \ 1
    // [0][0] 0 0 \
    //        0 0 0
    [[3, 4, 5], [-1, -1, -1], [-1, -1, -1], [-1, -1, -1]],
    // 5
    // [0][0] 0 0 0
    // [0][1] 0 0 /
    //        0 / 1
    [[5, 6, 7], [-1, -1, -1], [-1, -1, -1], [-1, -1, -1]],
    // 6
    // [1][0] 1 | 0
    // [1][0] 1 | 0
    //        1 | 0
    [[0, 2, 3], [0, 3, 7], [-1, -1, -1], [-1, -1, -1]],
    // 7
    // [0][0] 0 0 0
    // [1][1] - - -
    //        1 1 1
    [[0, 1, 5], [0, 5, 6], [-1, -1, -1], [-1, -1, -1]],
    // 8
    // [0][1] 0 / 1
    // [1][0] / 1 /
    //        1 / 0
    [[0, 1, 7], [1, 3, 7], [3, 6, 7], [3, 4, 5]],
    // 9
    // [1][0] 1 \ 0
    // [0][1] \ 1 \
    //        0 \ 1
    [[1, 2, 3], [1, 3, 7], [3, 5, 7], [5, 6, 7]],
    // 10
    // [1][1] 1 1 1
    // [0][0] - - -
    //        0 0 0
    [[1, 2, 4], [1, 4, 5], [-1, -1, -1], [-1, -1, -1]],
    // 11
    // [0][1] 0 | 1
    // [0][1] 0 | 1
    //        0 | 1
    [[0, 2,  3], [ 0, 3, 7], [-1, -1, -1], [-1, -1, -1]],
    // 12
    // [1][1] 1 1 1
    // [1][0] 1 1 /
    //        1 / 0
    [[0, 2, 7], [2, 5, 7], [2, 4, 5], [-1, -1, -1]],
    // 13
    // [1][0] 1 \ 0
    // [1][1] 1 1 \
    //        1 1 1
    [[0, 2, 3], [0, 3, 5], [0, 5, 6], [-1, -1, -1]],
    // 14
    // [0][1] 0 / 1
    // [1][1] / 1 1
    //        1 1 1
    [[1, 3, 6], [0, 1, 6], [3, 4, 6], [-1, -1, -1]],
    // 15
    // [1][1] 1 1 1
    // [0][1] \ 1 1
    //        0 \ 1
    [[1, 2, 4], [1, 4, 7], [4, 6, 7], [-1, -1, -1]],
    // 16
    // [1][1] 1 1 1
    // [1][1] 1 1 1
    //        1 1 1
    [[0, 2, 4], [0, 4, 6], [-1, -1, -1], [-1, -1, -1]],
];
