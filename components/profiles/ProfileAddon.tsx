import {ProfileAddon} from "../../lib/types.ts";

export default function ProfileAddonRow({addon,deleteFn}:Readonly<{addon:ProfileAddon,deleteFn:(addon:ProfileAddon)=>void}>){

    return(
        <div className={'w-full items-center flex justify-between bg-pink-300 my-1 p-2 first:mt-0 last:mb-0'}>
            {addon.name}
            <button className={'w-fit'} onClick={()=>deleteFn(addon)}>
                Delete
            </button>
        </div>
    )
}