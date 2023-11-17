/*
 Generated by typeshare 1.7.0
*/

export type SectionMeta = 
	| { type: "section", value: {
	value: string;
}}
	| { type: "repeat", value: {
	value: number;
}};

export type ChordInfoMeta = 
	| { type: "key", value: Key };

export type ChordExpression = 
	| { type: "chord", value: Chord }
	| { type: "unidentified", value?: undefined }
	| { type: "noChord", value?: undefined }
	| { type: "same", value?: undefined };

export interface ChordInfo {
	metaInfos: ChordInfoMeta[];
	chordExpression: ChordExpression;
	denominator?: string;
}

export type ChordBlock = ChordInfo[];

export interface Section {
	metaInfos: SectionMeta[];
	chordBlocks: ChordBlock[];
}

export type Ast = Section[];

export enum Base {
	A = "A",
	B = "B",
	C = "C",
	D = "D",
	E = "E",
	F = "F",
	G = "G",
}

export enum Accidental {
	Sharp = "#",
	Flat = "b",
}

export enum ChordType {
	Minor = "m",
	Major = "M",
	Augmented = "aug",
	Diminished = "dim",
}

export enum Extension {
	Two = "2",
	Three = "3",
	FlatThree = "b3",
	Four = "4",
	FlatFive = "b5",
	Five = "5",
	SharpFive = "#5",
	FlatSix = "b6",
	Six = "6",
	Seven = "7",
	FlatNine = "b9",
	Nine = "9",
	SharpNine = "#9",
	FlatEleven = "b11",
	Eleven = "11",
	SharpEleven = "#11",
	FlatThirteen = "b13",
	Thirteen = "13",
	SharpThirteen = "#13",
	MajorSeven = "M7",
	MajorNine = "M9",
	MajorEleven = "M11",
	MajorThirteen = "M13",
	Add9 = "add9",
	Add11 = "add11",
	Add13 = "add13",
	Sus2 = "sus2",
	Sus4 = "sus4",
	HalfDiminish = "o",
}

export interface ChordDetailed {
	base: Base;
	accidental?: Accidental;
	chordType: ChordType;
	extensions: Extension[];
}

export interface Chord {
	plain: string;
	detailed: ChordDetailed;
}

export enum Key {
	Cb_M = "Cb_M",
	Cb_m = "Cb_m",
	C_M = "C_M",
	C_m = "C_m",
	Cs_M = "Cs_M",
	Cs_m = "Cs_m",
	Db_M = "Db_M",
	Db_m = "Db_m",
	D_M = "D_M",
	D_m = "D_m",
	Ds_M = "Ds_M",
	Ds_m = "Ds_m",
	Eb_M = "Eb_M",
	Eb_m = "Eb_m",
	E_M = "E_M",
	E_m = "E_m",
	Fb_M = "Fb_M",
	Fb_m = "Fb_m",
	F_M = "F_M",
	F_m = "F_m",
	Fs_M = "Fs_M",
	Fs_m = "Fs_m",
	Gb_M = "Gb_M",
	Gb_m = "Gb_m",
	G_M = "G_M",
	G_m = "G_m",
	Gs_M = "Gs_M",
	Gs_m = "Gs_m",
	Ab_M = "Ab_M",
	Ab_m = "Ab_m",
	A_M = "A_M",
	A_m = "A_m",
	As_M = "As_M",
	As_m = "As_m",
	Bb_M = "Bb_M",
	Bb_m = "Bb_m",
	B_M = "B_M",
	B_m = "B_m",
}

