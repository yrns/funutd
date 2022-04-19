# FunUTD

## Fun Universal Texture Definitions

FunUTD is a 3-D [procedural texture](https://en.wikipedia.org/wiki/Procedural_texture) library running on the CPU.
This is an alpha version undergoing rapid development and may contain rough edges.

### Features

* Different tiling modes, including tiling of all 3 dimensions
* An endless supply of procedurally generated, self-describing volumetric textures
* Isotropic value noise, isotropic gradient noise and Voronoi bases
* Palette generation with Okhsv and Okhsl color spaces

## Basics

The type returned by texture generators is `Box<dyn Texture>`.
`Texture` is the trait implemented by procedural textures.

Data for procedural generation is contained in `Dna` objects.
Generator functions draw whatever data they need from the supplied `Dna` object.
`Dna` objects can be constructed full of random data from a seed value.

Textures can describe themself, that is, print the code that generates them.
This is done using the `get_code` method. Obtained codes can be copied and
pasted around and subjected to further scrutiny.

## Future

`Dna` objects can be mutated or crossed over to create variations of genotypes
or to optimize a texture for a purpose.
