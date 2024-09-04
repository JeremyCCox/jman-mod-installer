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

export interface ProfileAddon{
    name:string,
    fileName:string,
    location:string,
    versions:string[],
    dependencies:ProfileAddon[];
    addonType:string,
}