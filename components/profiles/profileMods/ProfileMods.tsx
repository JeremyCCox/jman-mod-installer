import  {useState} from "react";
import {ProfileAddon} from "../../../lib/types.ts";
import LoadingSpinner from "../../LoadingSpinner.tsx";
import FileInput from "../../inputs/FileInput.tsx";
import NewModRow from "../../ModInstaller/NewModRow.tsx";
import {invoke} from "@tauri-apps/api";
import {useSearchParams} from "react-router-dom";
import {useQueryClient} from "react-query";

class FilePath{
    path:string;
    constructor(filepath:string) {
        this.path = filepath;
    }
    getFileInfo(){
        console.log(this.path)
        let pathSplit = this.path.split("\\");
        let fileNameSplit = pathSplit[pathSplit.length-1].split(".")
        let extension;
        if(fileNameSplit.length > 1){
             extension = fileNameSplit.pop()
        }
        let name = fileNameSplit.join(".")
        return {name,fileName:[name,extension].join("."),extension}
    }
}

export default function ProfileMods({mods}:Readonly<{ mods?:ProfileAddon[] }>){


        const [newMods, setNewMods] = useState<ProfileAddon[]|undefined>(undefined)
    const [URLSearchParams] = useSearchParams()
    const queryClient = useQueryClient();
    const fileHandler=(files:string[])=>{
        let mods:ProfileAddon[] = [];
        for (let file of files){
            let filePath = new FilePath(file)
            let {name,fileName} = filePath.getFileInfo();
           mods.push({
                dependencies: [],
                fileName,
                location: filePath.path,
                name,
                versions: []
            })
        }
        setNewMods(mods)
    }
    const updateMod=(mod:ProfileAddon)=>{
        let nNewMods:ProfileAddon[] = Object.assign([mod],newMods)
        setNewMods(nNewMods)
    }
    const installNewMods=async () => {
        if(!newMods){
            return
        }
        let testList = []
        for (let mod of newMods){
            let depNames = mod.dependencies.map(dep=> {
                return (dep.name)
            });
            testList.push({
                addonType:"Mod",
                name:mod.name,
                fileName:mod.fileName,
                dependencies:depNames,
                location:mod.location,
                versions:mod.versions,
            })
        }
        await queryClient.invalidateQueries(["compare_profiles",URLSearchParams.get("profile")])
        await queryClient.invalidateQueries(["local-profiles",URLSearchParams.get("profile")])
        await invoke("install_new_mods", {modList: testList,profile:URLSearchParams.get("profile")})
        await queryClient.refetchQueries(["local-profiles",URLSearchParams.get("profile")])
        await queryClient.refetchQueries(["compare_profiles",URLSearchParams.get("profile")])
        setNewMods(undefined)
    }

    if(mods){
        return(
            <div className={'m-2'}>
                <div className={'flex flex-wrap relative'}>
                    <h3 className={'text-center font-bold text-2xl w-full '}>Mods</h3>
                    <FileInput fileHandler={fileHandler}/>
                </div>
                {newMods&&
                    <table className={'w-full border-black border-2 relative  '}>
                        <thead>
                            <tr>
                                <th>
                                    Name
                                </th>
                                <th>
                                    File name
                                </th>
                                <th>
                                    Dependencies
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                        {newMods.map(mod=>{
                            return (
                                <NewModRow key={`modrow-${mod.fileName}`} mod={mod} newMods={newMods} profileMods={mods} updateMod={updateMod}/>
                            )
                        })}
                        </tbody>
                    </table>
                }
                {newMods && newMods.length > 0&&
                    <button onClick={installNewMods}>
                        Install new mods
                    </button>
                }


                <div className={'max-h-60 border-2 border-black overflow-y-auto'}>
                    {mods.map(mod=>{
                        return(
                            <p className={''} key={mod.name}>
                                {mod.name}
                            </p>
                        )
                    })}
                </div>
            </div>
        )
    }else{
        return(
            <LoadingSpinner/>
            )
    }
}