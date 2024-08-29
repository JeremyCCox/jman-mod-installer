import React from "react";
import LoadingSpinner from "../../LoadingSpinner.tsx";

export default function ProfileMods({mods}:Readonly<{ mods:any }>){


    if(mods){

        return(
            <div className={'m-2'}>
                <h3 className={'text-center font-bold text-2xl'}>Mods</h3>
                <div className={'max-h-60 border-2 border-black overflow-y-auto'}>
                    {mods.map(mod=>{
                        return(
                            <p className={''} key={mod}>
                                {mod}
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