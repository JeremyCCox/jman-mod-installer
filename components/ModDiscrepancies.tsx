export default function ModDiscrepancies({modlist,notice, callback,callbackTitle}:Readonly<{modlist:string[],notice:string,callback:any,callbackTitle:string}>){
    return(
        <>
            <h3 className={'text-2xl text-red-400 text-center font-bold'}>
                {notice}
            </h3>
            <button onClick={callback}>
                {callbackTitle}
            </button>
            {modlist.map(mod=>{
                return(
                    <p className={'text-sm overflow-x-auto text-nowrap font-bold py-1 my-1'}>
                        {mod}
                    </p>
                )
            })}
        </>
    )
}