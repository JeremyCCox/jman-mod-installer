import LoadingSpinner from "./LoadingSpinner.tsx";
import {ProfileAddon} from "../lib/types.ts";

export default function ModDiscrepancies({modlist,notice, callback,callbackTitle,loading=false}:Readonly<{modlist:ProfileAddon[],notice:string,callback:any,callbackTitle:string,loading?:boolean}>){

    return(
        <>
            <h3 className={'text-2xl text-red-400 text-center font-bold'}>
                {notice}
            </h3>
            <button  onClick={callback} disabled={loading}>
                {loading?<LoadingSpinner/>:callbackTitle}
            </button>
            {modlist.map(mod=>{
                return(
                    <p className={'text-sm overflow-x-auto text-nowrap font-bold py-1 my-1'}>
                        {mod.name}
                    </p>
                )
            })}
        </>
    )
}