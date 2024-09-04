import {ProfileAddon} from "../../lib/types";
import {useState} from "react";
import {useQuery} from "react-query";
import {invoke} from "@tauri-apps/api";

export default function NewModRow({mod,newMods,profileMods,updateMod}:Readonly<{mod:ProfileAddon,newMods:ProfileAddon[],profileMods:ProfileAddon[],updateMod:(mod:ProfileAddon)=>void}>){
    const cleanNewMods = newMods.filter(({name})=>name !== mod.name)
    const cleanProfileMods = profileMods.filter(({name})=>name !== mod.name)
    const [toggleDep, setToggleDep] = useState(false);
    const allModsQuery=useQuery(["allMods"],async () => {
        return await invoke<ProfileAddon[]>("read_remote_mods")
    }, {enabled:toggleDep});

    const addDependency=(e:React.FormEvent<HTMLFormElement>)=>{
        e.preventDefault();

        let masterList = cleanNewMods;
        masterList.push(...cleanProfileMods)
        if(allModsQuery.data){
            let cleanMods = allModsQuery.data.filter(({name})=>name !== mod.name);
            masterList.push(...cleanMods)
        }
        let newMod = masterList[e.currentTarget['newDep'].selectedIndex-1];
        if(!mod.dependencies.find(({name})=>name === newMod.name) &&!(mod.name === newMod.name)){
            mod.dependencies.push(newMod);
            updateMod(mod);
        }
    }

    return(
        <tr className={'border-2 border-black'}>
            <td>{mod.name}</td>
            <td>{mod.fileName}</td>
            <td className={'grid relative'}>
                {mod.dependencies.map(dep=>{
                    return (<span>{dep.name}</span>)
                })}
                <button type={'button'} onClick={()=>{setToggleDep(!toggleDep)}}>{toggleDep?"Close Dependencies":"Add Dependency"}</button>
                {
                    toggleDep&&
                    <form onSubmit={addDependency} className={'absolute z-10 top-full w-full h-fit bg-blue-600 flex justify-between'}>
                        <select id={'newDep'}>
                            <option>

                            </option>
                            <optgroup label={'New mods'} >
                                {cleanNewMods.map(nMod=>{
                                    return(
                                        <option key={`nModOption-${nMod.fileName}`} value={nMod.name}>
                                            {nMod.name}
                                        </option>
                                    )
                                })}
                            </optgroup>
                            <optgroup label={'Current Profile mods'} >
                                {cleanProfileMods.map(pMod=>{
                                    return(
                                        <option key={`pModOption-${pMod.fileName}`} value={pMod.name}>
                                            {pMod.name}
                                        </option>
                                    )
                                })}
                            </optgroup>
                            <optgroup label={'ALL mods'} >
                                {allModsQuery.data?.map(aMod=>{
                                    if(aMod.name !== mod.name){
                                        return(
                                            <option  key={`aModOption-${aMod.fileName}`} value={aMod.name}>
                                                {aMod.name}
                                            </option>
                                        )
                                    }
                                })}
                            </optgroup>
                        </select>
                        <button className={'w-fit'} > Add</button>
                    </form>
                }

            </td>
        </tr>
    )
}