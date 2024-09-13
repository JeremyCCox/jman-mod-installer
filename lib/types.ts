import {UseQueryResult} from "react-query";

export interface LauncherProfile{
    created?:string,
    game_dir?:string,
    icon?:string,
    lastVersionId?:string,
    name?:string,
}
export interface RemoteProfile{
    name:string,
    mods?:ProfileAddon[],
    launcherProfile?:LauncherProfile;
    resourcePacks?:ProfileAddon[],
}
export interface LocalProfile{
    name:string,
    mods?:ProfileAddon[],
    launcherProfile?:LauncherProfile;
    resourcePacks?:ProfileAddon[],
}
export interface InstallerProfile{
    defaultGameDir:string,
    sftpServer?:string,
    sftpPort?:number,
    sftpUsername?:string,
    sftpPassword?:string
}
export enum AddonType{
    Mod="Mod",
    ResourcePack="ResourcePack",
}
export interface ProfileAddon{
    name:string,
    fileName:string,
    location:string,
    versions:string[],
    dependencies:string[];
    addonType:string,
}
export interface ConfigQuery{
    accessQuery:UseQueryResult<{success:boolean}>,
    configQuery:UseQueryResult<InstallerProfile>,
    attemptLogin:(config:InstallerProfile)=>Promise<unknown>,
    logout:()=>void,
}