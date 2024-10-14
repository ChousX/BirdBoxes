use bevy::prelude::*;
use bevy::tasks::ParallelIterator;
use bevy::{
    render::{
        mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology
    }, sprite::Mesh2dHandle, utils::HashMap,
    log::info,
};

pub struct BirdBoxesPlugin;
impl Plugin for BirdBoxesPlugin{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkSize>()
            .init_resource::<IsoLevel>()
            .init_resource::<IsoDistance>()
            .add_systems(PreUpdate, (add_mesh, update_mesh).chain());
    }
}

///The Size Of the IsoField
#[derive(Resource, Debug)]
pub struct ChunkSize(pub u32, pub u32);
impl Default for ChunkSize{
    fn default() -> Self{
        Self(2, 2)
    }
}

///The threshold when 
#[derive(Resource, Debug)]
pub struct IsoLevel(pub f32);
impl Default for IsoLevel{
    fn default() -> Self{
        Self(1.0)
    }
}

#[derive(Resource, Debug)]
pub struct IsoDistance(pub f32);
impl Default for IsoDistance{
    fn default() -> Self{
        Self(1.0)
    }
}

/////////

fn add_mesh(
    mut commands: Commands,
    iso_field_q: Query<(&IsoField, Entity), Without<Mesh2dHandle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_size: Res<ChunkSize>,
    iso_level: Res<IsoLevel>,
    iso_distance: Res<IsoDistance>,
){
    for (field, entity) in iso_field_q.iter(){
        info!("New Mesh");
        let mesh_2d = Mesh2dHandle(meshes.add(field
                    .sample_all()
                    .build_mesh(iso_distance.0, iso_level.0)));
        commands.entity(entity).insert(mesh_2d);
    }
}

fn update_mesh(
    iso_field_q: Query<(&IsoField, &Mesh2dHandle), Changed<IsoField>>,
    mut meshes: ResMut<Assets<Mesh>>,
    iso_level: Res<IsoLevel>,
    iso_distance: Res<IsoDistance>,
){
    for (iso_field, mesh_2d) in iso_field_q.iter(){
        info!("Mesh Update");
        let mesh = meshes.add(iso_field
                .sample_all()
                .build_mesh(iso_distance.0, iso_level.0));
        if let Some(mut stored_mesh) = meshes.get_mut(mesh_2d.0.clone()){

        }
    }
}

#[derive(Bundle, Default)]
pub struct BirdBoxeBundle<M: Asset>{
    pub iso_field: IsoField,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub material: Handle<M>,
    pub view_visibility: ViewVisibility,
}

#[derive(Component, Default)]
pub struct IsoField{
    x_size: usize,
    field: Vec<f32>
}

type Size = (usize, usize);

//New iso field fn's
impl IsoField {
    pub fn new(size: impl Into<Size>) -> Self{
        let (x, y): Size = size.into();
        let field = vec![0.0; x * y];
        Self{
            x_size: x,
            field
        }
    }

    pub fn new_from(size: impl Into<Size>, vec: Vec<f32>) -> Self {
        let (x, y): Size = size.into();
        #[cfg(debug_assertions)]
        if vec.len() % x != 0 {
            panic!("vec len and size do not match");
        }
        Self{
            x_size: x,
            field: vec
        }
    }
}

//getters and setters
impl IsoField {
    pub fn get(&self, x: usize, y: usize) -> f32{
        self.field[self.index(x, y)]
    }
    pub fn set(&mut self, x: usize, y: usize, val: f32){
        let index = self.index(x, y);
        self.field[index] = val
    }
    pub fn sample(&self, x: usize, y: usize) -> IsoSample{
        let sample = [
                    self.get(x, y ), // bottom left
                    self.get(x, y + 1), // top left
                    self.get(x + 1, y + 1), // top right
                    self.get(x + 1, y), // bottom right
        ];
        IsoSample(sample)
    }
    pub fn sample_all(&self) -> IsoSamples {
        let mut samples = Vec::new();
        let y_size = self.field.len() / self.x_size;
        for y in 0..(y_size - 1){
            for x in 0..(self.x_size - 1){
                samples.push((self.sample(x, y), x, y));
            }
        }
        IsoSamples(samples)
    }
    fn index(&self, x: usize, y: usize) -> usize{
        y * self.x_size as usize + x
    }
}


pub struct IsoSamples(Vec<(IsoSample, usize, usize)>);
impl Iterator for IsoSamples {
    type Item = (IsoSample, usize, usize);
    fn next(&mut self) -> Option<Self::Item>{
        self.0.pop()
    }
}

impl IsoSamples {
    fn build_mesh(self, iso_distance: f32, iso_level: f32) -> Mesh {
        info!("Building Mesh[\n   iso_distance:{iso_distance}\n   iso_level:{iso_level}\n]");
        let mut used_indices = HashMap::<HashAbleVec2, usize>::new();
        let mut vertexes = Vec::<Vec3>::new();
        let mut indices = Vec::<u32>::new();
        let mut normals = Vec::<Vec3>::new();
        //let mut face_count = Vec::<usize>::new();
        let mut uvs = Vec::<Vec2>::new();
        'a:for (sample, x, y) in self{
            for tri in sample.to_tri_list(0.5){
                for tri_index in tri{
                    if let Some(vertex) = tri_index_to_vertex(tri_index){
                        let vertex = {
                            let x_off = x as f32 * iso_distance;
                            let y_off = y as f32 * iso_distance;
                            Vec2::new(x_off, y_off) + vertex * iso_distance
                        };
                        let h_vertex = HashAbleVec2::from(vertex.clone());
                        if let Some(indice) = used_indices.get(&h_vertex){
                            indices.push(*indice as u32);
                        } else {
                            // add vertex
                            let indice = vertexes.len();
                            used_indices.insert(h_vertex, indice);
                            vertexes.push(vertex.extend(0.0));
                            indices.push(indice as u32);
                            normals.push(Vec3::new(0.0, 0.0, 1.0));
                            uvs.push(Vec2::new(0.0, 0.0));
                            //face_count.push(0);
                        }
                    } else {
                        //continue 'a;
                    }
                }
            }
        }
        /*
        for i in (0..(indices.len())).step_by(3) {
            //getting the indices
            let (i0, i1, i2) = (
                indices[i] as usize,
                indices[i + 1] as usize,
                indices[i + 2] as usize,
            );

            //gettting the vertexes
            let (v0, v1, v2) = (
                vertexes[i0],    
                vertexes[i1],    
                vertexes[i2],    
            );

            //calculating the normal
            let normal = (v1 - v0).cross(v2 - v0).normalize();

            //adding the normal to all the indice
            normals[i0] += normal;
            normals[i1] += normal;
            normals[i2] += normal;

            //add one to the face counter for later
            face_count[i0] += 1;
            face_count[i1] += 1;
            face_count[i2] += 1;
        }
        
        //finishing the normals
        for i in 0..vertexes.len(){
            normals[i] /= face_count[i] as f32;
        }
        */

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertexes)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices))
    }
}

pub struct IsoSample([f32; 4]);
impl IsoSample{
    pub fn to_case(self, iso_level: f32) -> u8{
        const MASK: [u8; 4] = [ 1, 2, 4, 8];
        let mut out = 0;
        for (i, f) in self.0.iter().enumerate(){
            if *f > iso_level {
                out = out | MASK[i];
            }
        }
    dbg!(out)
    }

    pub fn to_tri_list(self, iso_level: f32) -> [[i8; 3]; 4]{
        let case = self.to_case(iso_level) as usize;
        CASE_TABLE[case]
    }
}

fn tri_index_to_vertex(index: i8) -> Option<Vec2>{
    Some(match index {
        -1 => {return None;},
        0 => Vec2::new(0.0, 0.0),
        1 => Vec2::new(0.0, 0.5),
        2 => Vec2::new(0.0, 1.0),
        3 => Vec2::new(0.5, 1.0),
        4 => Vec2::new(1.0, 1.0),
        5 => Vec2::new(1.0, 0.5),
        6 => Vec2::new(1.0, 0.0),
        7 => Vec2::new(0.5, 0.0),
        _ => unreachable!()
    })
}

#[derive(Hash, Eq, PartialEq)]
struct HashAbleVec2{
    x: (u32, i16, i8),
    y: (u32, i16, i8),
}

impl From<Vec2> for HashAbleVec2{
    fn from(val: Vec2) -> Self{
        Self { x: integer_decode(val.x), y: integer_decode(val.y) }
    }
}

fn integer_decode(val: f32) -> (u32, i16, i8) {
    let bits: u32 = unsafe { std::mem::transmute(val) };
    let sign: i8 = if bits >> 31 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0x7fffff) << 1
    } else {
        (bits & 0x7fffff) | 0x800000
    };

    exponent -= 127 + 23;
    (mantissa, exponent, sign)
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
