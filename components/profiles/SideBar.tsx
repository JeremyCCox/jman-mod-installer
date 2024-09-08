import ProfileList from "./ProfileList.tsx";
import React, {useState} from "react";
import {useSearchParams} from "react-router-dom";
import {useQuery, UseQueryResult} from "react-query";
import {invoke} from "@tauri-apps/api";

export default function SideBar(){
    const [toggle]=useState(true)
    // @ts-ignore
    const [searchParams,setSearchParams] = useSearchParams();
    let local_profiles:UseQueryResult<[string]> = useQuery(["list_local_profiles"],async () => {
        return await invoke<[string]>("list_local_profiles").then(res=>{
            return res;
        }).catch(err=>{
            console.error(err)
            return([])
        })
    })
    let remote_profiles:UseQueryResult<[string]> = useQuery(["list_remote_profiles"],async () => {
        return await invoke<[string]>("list_remote_profiles").then(res=>{
            return res;
        }).catch(err=>{
            console.error(err)
            return([])
        })
    })
    const selectProfile=(e:React.MouseEvent<HTMLButtonElement>)=>{
        // setSearchParams({})
        console.log("adad")
        setSearchParams({ "source":e.currentTarget.name.split(":")[0],"profile": e.currentTarget.name.split(":")[1],"page":"profile"});
    }
    //e:React.MouseEvent<HTMLButtonElement>
    const selectResourcePacks=()=>{
        setSearchParams({"page":"resourcepack"})
    }
    return(
        <div className={'bg-red-500'}>
            {/*<div>*/}
            {/*    <button className={'px-0 mx-0 float-right'} onClick={()=>{setToggle(!toggle)}}>{toggle?" - ":" + "}</button>*/}
            {/*</div>*/}
            <div className={'grid overflow-y-auto transition-all duration-700'} style={{width:toggle?200:0}}>
                <ProfileList title={"Local Profiles"} profile_query={local_profiles} selectProfile={selectProfile} source={"local"}/>
                <ProfileList title={"Remote Profiles"} profile_query={remote_profiles} selectProfile={selectProfile} source={"remote"}/>
                <div>
                    <h3 className={'text-2xl text-center font-bold p-2 bg-gray-700'}>Other</h3>
                    <button type={"button"} onClick={selectResourcePacks}>
                        Resource Packs
                    </button>
                </div>
            </div>
        </div>
        )
}